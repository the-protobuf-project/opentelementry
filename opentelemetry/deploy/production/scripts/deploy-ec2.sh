#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROD_DIR="$(dirname "$SCRIPT_DIR")"
OTEL_DIR="$(dirname "$(dirname "$PROD_DIR")")"

# Load .env if exists
if [ -f "$PROD_DIR/.env" ]; then
    source "$PROD_DIR/.env"
fi

# Use environment variables with defaults
SSH_KEY="${SSH_KEY_PATH:-~/.ssh/id_ed25519}"
DOMAIN="${DOMAIN:-telemetry.example.com}"
GRAFANA_USER="${GRAFANA_ADMIN_USER:-admin}"
GRAFANA_PASS="${GRAFANA_ADMIN_PASSWORD:-changeme}"

echo "=== Pulse Telemetry EC2 Deployment ==="

# Get public IP from terraform output or argument
if [ -n "$1" ]; then
    PUBLIC_IP="$1"
else
    cd "$PROD_DIR/terraform"
    PUBLIC_IP=$(terraform output -raw public_ip 2>/dev/null || echo "")
    cd "$PROD_DIR"
fi

if [ -z "$PUBLIC_IP" ]; then
    echo "Error: No public IP found. Either:"
    echo "  1. Run 'terraform apply' first in terraform/"
    echo "  2. Pass IP as argument: ./deploy-ec2.sh <PUBLIC_IP>"
    exit 1
fi

echo "Deploying to: $PUBLIC_IP"
echo "SSH Key: $SSH_KEY"

# Wait for instance to be ready
echo "Waiting for instance to be ready..."
for i in {1..30}; do
    if ssh -i "$SSH_KEY" -o ConnectTimeout=5 -o StrictHostKeyChecking=no ec2-user@"$PUBLIC_IP" "echo ready" 2>/dev/null; then
        echo "Instance is ready!"
        break
    fi
    echo "  Waiting... ($i/30)"
    sleep 10
done

# Install Docker and Docker Compose if not present
echo "Ensuring Docker is installed..."
ssh -i "$SSH_KEY" ec2-user@"$PUBLIC_IP" "
    if ! command -v docker &> /dev/null; then
        sudo dnf update -y
        sudo dnf install -y docker
        sudo systemctl enable docker
        sudo systemctl start docker
        sudo usermod -aG docker ec2-user
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        sudo curl -L 'https://github.com/docker/compose/releases/latest/download/docker-compose-linux-x86_64' -o /usr/local/bin/docker-compose
        sudo chmod +x /usr/local/bin/docker-compose
        sudo ln -sf /usr/local/bin/docker-compose /usr/bin/docker-compose
    fi
    
    # Install Docker Buildx
    sudo mkdir -p /usr/local/lib/docker/cli-plugins
    if [ ! -f /usr/local/lib/docker/cli-plugins/docker-buildx ]; then
        sudo curl -SL https://github.com/docker/buildx/releases/download/v0.17.1/buildx-v0.17.1.linux-amd64 -o /usr/local/lib/docker/cli-plugins/docker-buildx
        sudo chmod +x /usr/local/lib/docker/cli-plugins/docker-buildx
    fi
"

# Create directories
echo "Creating directories..."
ssh -i "$SSH_KEY" ec2-user@"$PUBLIC_IP" "sudo mkdir -p /opt/pulse/{config,envoy,dashboards,docker,certs} && sudo chown -R ec2-user:ec2-user /opt/pulse"

# Copy production files
echo "Copying production files..."
scp -i "$SSH_KEY" -r "$PROD_DIR/config/"* ec2-user@"$PUBLIC_IP":/opt/pulse/config/
scp -i "$SSH_KEY" -r "$PROD_DIR/envoy/"* ec2-user@"$PUBLIC_IP":/opt/pulse/envoy/
scp -i "$SSH_KEY" "$PROD_DIR/docker-compose.prod.yaml" ec2-user@"$PUBLIC_IP":/opt/pulse/

# Copy dashboards and docker build files
scp -i "$SSH_KEY" -r "$OTEL_DIR/dashboards/"* ec2-user@"$PUBLIC_IP":/opt/pulse/dashboards/
scp -i "$SSH_KEY" -r "$OTEL_DIR/docker/"* ec2-user@"$PUBLIC_IP":/opt/pulse/docker/

# Generate self-signed certs if not exist
echo "Setting up TLS certificates..."
ssh -i "$SSH_KEY" ec2-user@"$PUBLIC_IP" "
    if [ ! -f /opt/pulse/certs/fullchain.pem ]; then
        openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
            -keyout /opt/pulse/certs/privkey.pem \
            -out /opt/pulse/certs/fullchain.pem \
            -subj '/CN=$DOMAIN'
        chmod 644 /opt/pulse/certs/privkey.pem
    fi
"

# Create .env file
echo "Creating .env file..."
ssh -i "$SSH_KEY" ec2-user@"$PUBLIC_IP" "cat > /opt/pulse/.env << 'EOF'
GRAFANA_ADMIN_USER=$GRAFANA_USER
GRAFANA_ADMIN_PASSWORD=$GRAFANA_PASS
DOMAIN=$DOMAIN
EOF"

# Start services
echo "Starting services..."
ssh -i "$SSH_KEY" ec2-user@"$PUBLIC_IP" "cd /opt/pulse && sudo docker-compose -f docker-compose.prod.yaml up -d --build"

# Wait for services to be healthy
echo "Waiting for services to start..."
sleep 10

# Check status
echo ""
echo "Service status:"
ssh -i "$SSH_KEY" ec2-user@"$PUBLIC_IP" "sudo docker ps --format 'table {{.Names}}\t{{.Status}}'"

echo ""
echo "=== Deployment Complete ==="
echo "Public IP: $PUBLIC_IP"
echo ""
echo "Access:"
echo "  Dashboard: https://$PUBLIC_IP (accept self-signed cert)"
echo "  After DNS: https://$DOMAIN"
echo "  Credentials: $GRAFANA_USER / $GRAFANA_PASS"
echo ""
echo "OTLP Endpoints:"
echo "  gRPC: $PUBLIC_IP:4317"
echo "  HTTP: $PUBLIC_IP:4318"
echo ""
echo "DNS: Create A record: $DOMAIN -> $PUBLIC_IP"
echo ""
echo "SSH: ssh -i $SSH_KEY ec2-user@$PUBLIC_IP"
