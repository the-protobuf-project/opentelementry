# EC2-only deployment for Pulse Telemetry
# NO EKS = saves $73/month!
# Cost: t3.medium spot ~$10-15/month + EIP ~$3.60/month = ~$15-20/month

terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.92"
    }
  }
  required_version = ">= 1.2"
}

provider "aws" {
  region = var.aws_region
}

variable "aws_region" {
  description = "AWS region"
  default     = "us-east-1"
}

variable "instance_type" {
  description = "EC2 instance type (t3.medium = 2 vCPU, 4GB RAM)"
  default     = "t3.medium"
}

variable "domain_name" {
  description = "Domain for telemetry dashboard"
  default     = "telemetry.example.com"
}

variable "grafana_password" {
  description = "Grafana admin password"
  default     = "changeme"
  sensitive   = true
}

variable "vpc_id" {
  description = "VPC ID to deploy into (leave empty to use default VPC)"
  default     = ""
}

variable "subnet_id" {
  description = "Subnet ID to deploy into (leave empty to auto-select)"
  default     = ""
}

# Use specified VPC or default
data "aws_vpc" "selected" {
  id = var.vpc_id != "" ? var.vpc_id : null
  default = var.vpc_id == "" ? true : null
}

data "aws_subnets" "default" {
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.selected.id]
  }
}

locals {
  subnet_id = var.subnet_id != "" ? var.subnet_id : data.aws_subnets.default.ids[0]
}

# Security group
resource "aws_security_group" "pulse" {
  name        = "pulse-telemetry-sg"
  description = "Pulse telemetry security group"
  vpc_id      = data.aws_vpc.selected.id

  # SSH
  ingress {
    description = "SSH"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # HTTP (redirect to HTTPS)
  ingress {
    description = "HTTP"
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # HTTPS
  ingress {
    description = "HTTPS"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # OTLP gRPC
  ingress {
    description = "OTLP gRPC"
    from_port   = 4317
    to_port     = 4317
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # OTLP HTTP
  ingress {
    description = "OTLP HTTP"
    from_port   = 4318
    to_port     = 4318
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Grafana direct access (backup)
  ingress {
    description = "Grafana"
    from_port   = 3000
    to_port     = 3000
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "pulse-telemetry"
  }
}

# Latest Amazon Linux 2023 AMI
data "aws_ami" "al2023" {
  most_recent = true
  owners      = ["amazon"]

  filter {
    name   = "name"
    values = ["al2023-ami-*-x86_64"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

variable "ssh_public_key_content" {
  description = "SSH public key content for EC2 access"
  type        = string
}

# SSH Key
resource "aws_key_pair" "pulse" {
  key_name   = "pulse-telemetry-key"
  public_key = var.ssh_public_key_content
}

# User data script - installs Docker (files copied via SSH after)
locals {
  user_data = <<-EOF
    #!/bin/bash
    set -ex
    
    # Update system
    dnf update -y
    
    # Install Docker
    dnf install -y docker git
    systemctl enable docker
    systemctl start docker
    usermod -aG docker ec2-user
    
    # Install Docker Compose
    curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-linux-x86_64" -o /usr/local/bin/docker-compose
    chmod +x /usr/local/bin/docker-compose
    ln -sf /usr/local/bin/docker-compose /usr/bin/docker-compose
    
    # Create pulse directory structure
    mkdir -p /opt/pulse/{config,certs,envoy,dashboards}
    
    # Generate self-signed certs (replace with Let's Encrypt later)
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
      -keyout /opt/pulse/certs/privkey.pem \
      -out /opt/pulse/certs/fullchain.pem \
      -subj "/CN=${var.domain_name}"
    
    # Set permissions
    chown -R ec2-user:ec2-user /opt/pulse
    
    echo "Setup complete! Copy files via SCP then run docker-compose"
  EOF
}

# Spot instance request for lowest cost
resource "aws_spot_instance_request" "pulse" {
  ami                    = data.aws_ami.al2023.id
  instance_type          = var.instance_type
  key_name               = aws_key_pair.pulse.key_name
  vpc_security_group_ids = [aws_security_group.pulse.id]
  subnet_id              = local.subnet_id

  spot_type                      = "persistent"
  instance_interruption_behavior = "stop"
  wait_for_fulfillment           = true

  user_data = base64encode(local.user_data)

  root_block_device {
    volume_size           = 30
    volume_type           = "gp3"
    delete_on_termination = true
  }

  tags = {
    Name = "pulse-telemetry"
  }
}

# Elastic IP for stable DNS pointing
resource "aws_eip" "pulse" {
  domain = "vpc"

  tags = {
    Name = "pulse-telemetry"
  }
}

# Associate EIP with spot instance
resource "aws_eip_association" "pulse" {
  instance_id   = aws_spot_instance_request.pulse.spot_instance_id
  allocation_id = aws_eip.pulse.id
}

# Outputs
output "public_ip" {
  description = "Public IP address - point DNS here"
  value       = aws_eip.pulse.public_ip
}

output "ssh_command" {
  description = "SSH into the instance"
  value       = "ssh ec2-user@${aws_eip.pulse.public_ip}"
}

output "deploy_command" {
  description = "Run this after terraform apply to deploy services"
  value       = "cd .. && ./scripts/deploy-ec2.sh ${aws_eip.pulse.public_ip}"
}

output "dashboard_url" {
  description = "Dashboard URL (after DNS is configured)"
  value       = "https://${var.domain_name}"
}

output "dns_instructions" {
  description = "DNS configuration"
  value       = "Create an A record: ${var.domain_name} -> ${aws_eip.pulse.public_ip}"
}

output "monthly_cost_estimate" {
  description = "Estimated monthly cost"
  value       = "~$15-20/month (t3.medium spot ~$10-15 + EIP $3.60)"
}
