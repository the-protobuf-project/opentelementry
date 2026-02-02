# Pulse Telemetry - Production Deployment

Deploy the Pulse observability stack to AWS EC2 with Docker Compose.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         EC2 Instance                            │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    Envoy Proxy                            │  │
│  │         :80 (HTTP→HTTPS) :443 (HTTPS)                     │  │
│  │              :4317 (OTLP gRPC) :4318 (OTLP HTTP)          │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────┼───────────────────────────────┐  │
│  │                    Docker Network                         │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐  │  │
│  │  │  Loki   │  │  Tempo  │  │Prometheus│  │  Pyroscope  │  │  │
│  │  │  Logs   │  │ Traces  │  │ Metrics  │  │  Profiles   │  │  │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬──────┘  │  │
│  │       │            │            │              │          │  │
│  │       └────────────┴─────┬──────┴──────────────┘          │  │
│  │                          │                                │  │
│  │                   ┌──────┴──────┐                         │  │
│  │                   │   Grafana   │                         │  │
│  │                   │  Dashboard  │                         │  │
│  │                   └─────────────┘                         │  │
│  │                          │                                │  │
│  │                   ┌──────┴──────┐                         │  │
│  │                   │    OTEL     │                         │  │
│  │                   │  Collector  │                         │  │
│  │                   └─────────────┘                         │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Cost

- **EC2 t3.medium spot**: ~$10-15/month
- **Elastic IP**: ~$3.60/month
- **Total**: ~$15-20/month

No EKS control plane = **$73/month saved!**

## Prerequisites

- AWS CLI configured with credentials
- Terraform >= 1.2
- SSH key pair

## Quick Start

### 1. Configure Environment

```bash
cd deploy/production

# Copy and edit environment file
cp .env.example .env
nano .env  # Set your credentials
```

Required variables in `.env`:
```bash
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=your-secure-password
DOMAIN=telemetry.yourdomain.com
SSH_KEY_PATH=~/.ssh/id_ed25519
SSH_PUBLIC_KEY=ssh-ed25519 AAAA... your-email@example.com
```

### 2. Configure Terraform

```bash
cd terraform

# Copy and edit terraform variables
cp terraform.tfvars.example terraform.tfvars
nano terraform.tfvars
```

Required variables in `terraform.tfvars`:
```hcl
aws_region             = "us-east-1"
domain_name            = "telemetry.yourdomain.com"
ssh_public_key_content = "ssh-ed25519 AAAA... your-email@example.com"
```

### 3. Deploy Infrastructure

```bash
terraform init
terraform apply
```

### 4. Deploy Services

```bash
cd ..
./scripts/deploy-ec2.sh
```

### 5. Configure DNS

Add an A record pointing your domain to the EC2 public IP:
```
telemetry.yourdomain.com -> <PUBLIC_IP>
```

## Scripts

| Script | Description |
|--------|-------------|
| `scripts/deploy-ec2.sh` | Full deployment (first time) |
| `scripts/redeploy-ec2.sh` | Update existing deployment |
| `scripts/destroy-ec2.sh` | Destroy all infrastructure |
| `scripts/deploy-docker.sh` | Local deployment (no AWS) |
| `scripts/start-local-webhook.sh` | Local alert notifications |

## Access

After deployment:

| Service | URL |
|---------|-----|
| Dashboard | `https://your-domain.com` |
| Grafana (direct) | `http://<IP>:3000` |
| OTLP gRPC | `<IP>:4317` |
| OTLP HTTP | `<IP>:4318` |

Default credentials: `admin` / (password from .env)

## Sending Telemetry

Configure your applications to send telemetry to:

```bash
# OTLP gRPC
export OTEL_EXPORTER_OTLP_ENDPOINT=https://telemetry.yourdomain.com:4317

# OTLP HTTP
export OTEL_EXPORTER_OTLP_ENDPOINT=https://telemetry.yourdomain.com:4318
```

## TLS Certificates

By default, self-signed certificates are generated. For production:

### Let's Encrypt (recommended)

SSH into the instance and run:
```bash
sudo dnf install -y certbot
sudo certbot certonly --standalone -d telemetry.yourdomain.com
sudo cp /etc/letsencrypt/live/telemetry.yourdomain.com/fullchain.pem /opt/pulse/certs/
sudo cp /etc/letsencrypt/live/telemetry.yourdomain.com/privkey.pem /opt/pulse/certs/
cd /opt/pulse && sudo docker-compose -f docker-compose.prod.yaml restart envoy
```

## Alerting

Alertmanager is configured to send alerts via webhook. To receive desktop notifications:

```bash
# On your local machine
./scripts/start-local-webhook.sh
```

Configure your local IP in `config/alertmanager.yaml`.

## Updating

To update the deployment after code changes:

```bash
./scripts/redeploy-ec2.sh
```

## Troubleshooting

### Check service status
```bash
ssh ec2-user@<IP> "sudo docker ps"
```

### View logs
```bash
ssh ec2-user@<IP> "sudo docker logs pulse-prod-grafana-1"
```

### Restart all services
```bash
ssh ec2-user@<IP> "cd /opt/pulse && sudo docker-compose -f docker-compose.prod.yaml restart"
```

## Security Notes

- Change default Grafana password immediately
- Restrict SSH access to your IP in security group
- Use Let's Encrypt for production TLS
- Keep `.env` and `terraform.tfvars` out of version control
