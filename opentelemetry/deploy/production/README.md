# Pulse Telemetry - Production Deployment

An open-source observability stack with **Logs**, **Traces**, **Metrics**, and **Profiles** - all in one place.

Deploy to AWS EC2, EKS, or run locally with Docker Compose.

## Features

- **Grafana** - Unified dashboard for all telemetry
- **Loki** - Log aggregation
- **Tempo** - Distributed tracing
- **Prometheus** - Metrics collection
- **Pyroscope** - Continuous profiling
- **OpenTelemetry Collector** - Unified telemetry ingestion
- **Alertmanager** - Alert routing and notifications
- **Envoy Proxy** - TLS termination and routing

## Architecture

```
                    ┌─────────────────────────────────────┐
                    │           Your Applications         │
                    └──────────────┬──────────────────────┘
                                   │ OTLP (traces, logs, metrics)
                                   ▼
┌──────────────────────────────────────────────────────────────────┐
│  otel.yourdomain.com:443        │  telemetry.yourdomain.com:443  │
│  (OTLP ingestion + auth)        │  (Grafana dashboard)           │
├─────────────────────────────────┴────────────────────────────────┤
│                         Envoy Proxy                              │
│              TLS termination, routing, auth                      │
├──────────────────────────────────────────────────────────────────┤
│                      Docker Network                              │
│  ┌─────────┐  ┌─────────┐  ┌──────────┐  ┌──────────┐           │
│  │  Loki   │  │  Tempo  │  │Prometheus│  │Pyroscope │           │
│  │  Logs   │  │ Traces  │  │ Metrics  │  │ Profiles │           │
│  └────┬────┘  └────┬────┘  └────┬─────┘  └────┬─────┘           │
│       └────────────┴────────────┴─────────────┘                  │
│                         │                                        │
│              ┌──────────┴──────────┐                             │
│              │   OTEL Collector    │◄── Bearer Token Auth        │
│              └──────────┬──────────┘                             │
│                         │                                        │
│              ┌──────────┴──────────┐                             │
│              │      Grafana        │◄── Username/Password        │
│              └─────────────────────┘                             │
└──────────────────────────────────────────────────────────────────┘
```

## Deployment Options

| Option | Best For | Cost |
|--------|----------|------|
| **EC2** | Production, single instance | ~$15-20/month |
| **EKS** | Production, scalable | ~$90+/month |
| **Docker** | Local development | Free |

## Quick Start

### Option 1: Local Docker (Development)

```bash
cd deploy/production

# Configure
cp .env.example .env
nano .env  # Set your credentials

# Deploy
./scripts/deploy-docker.sh
```

Access at `https://localhost`

### Option 2: AWS EC2 (Production)

```bash
cd deploy/production

# 1. Configure environment
cp .env.example .env
nano .env

# 2. Configure Terraform
cd terraform
cp terraform.tfvars.example terraform.tfvars
nano terraform.tfvars

# 3. Deploy infrastructure
terraform init
terraform apply

# 4. Deploy services
cd ..
./scripts/deploy-ec2.sh
```

### Option 3: AWS EKS (Scalable)

```bash
cd deploy/production

# Configure
cp .env.example .env
nano .env

# Deploy (requires existing EKS cluster)
./scripts/deploy-eks.sh
```

## Configuration

### Required Environment Variables

```bash
# .env file
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=your-secure-password
DOMAIN=telemetry.yourdomain.com
OTEL_DOMAIN=otel.yourdomain.com
```

### DNS Configuration

Create A records pointing to your server IP:
```
telemetry.yourdomain.com -> <YOUR_IP>
otel.yourdomain.com      -> <YOUR_IP>
```

## Sending Telemetry

### OTLP Endpoints

| Protocol | Endpoint |
|----------|----------|
| HTTPS (recommended) | `https://otel.yourdomain.com/v1/traces` |
| gRPC | `otel.yourdomain.com:4317` |
| HTTP | `otel.yourdomain.com:4318` |

### Authentication

All OTLP requests require a Bearer token:

```bash
Authorization: Bearer <your-token>
```

Generate a token:
```bash
./scripts/setup-otel-endpoint.sh
```

### Client Examples

**Go (pulse-go):**
```go
pulseOpts := options.PulseOptions{
    Telemetry: options.DefaultTelemetry(),
}
pulseOpts.Telemetry.OTLP.Enabled = true
pulseOpts.Telemetry.OTLP.Host = "otel.yourdomain.com"
pulseOpts.Telemetry.OTLP.Port = 443
pulseOpts.Telemetry.OTLP.Secure = true
pulseOpts.Telemetry.OTLP.UseHTTP = true

// Set token via environment variable
// export OTEL_EXPORTER_OTLP_HEADERS="Authorization=Bearer <token>"
```

**Python:**
```python
from opentelemetry.exporter.otlp.proto.http.trace_exporter import OTLPSpanExporter

exporter = OTLPSpanExporter(
    endpoint="https://otel.yourdomain.com/v1/traces",
    headers={"Authorization": "Bearer <your-token>"}
)
```

**Node.js:**
```javascript
const { OTLPTraceExporter } = require('@opentelemetry/exporter-trace-otlp-http');

const exporter = new OTLPTraceExporter({
  url: 'https://otel.yourdomain.com/v1/traces',
  headers: {
    'Authorization': 'Bearer <your-token>'
  }
});
```

**Environment Variables (any OTEL SDK):**
```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=https://otel.yourdomain.com
export OTEL_EXPORTER_OTLP_HEADERS="Authorization=Bearer <your-token>"
```

## Scripts

| Script | Description |
|--------|-------------|
| `scripts/deploy-docker.sh` | Local deployment with Docker |
| `scripts/deploy-ec2.sh` | Full EC2 deployment |
| `scripts/deploy-eks.sh` | EKS/Kubernetes deployment |
| `scripts/redeploy-ec2.sh` | Update existing EC2 deployment |
| `scripts/destroy-ec2.sh` | Destroy EC2 infrastructure |
| `scripts/setup-otel-endpoint.sh` | Generate OTLP token and show client config |
| `scripts/start-local-webhook.sh` | Local alert notifications |

## TLS Certificates

### Self-Signed (Development)

Generated automatically on first deployment.

### Let's Encrypt (Production)

SSH into the instance and run:
```bash
# Stop Envoy temporarily
sudo docker stop pulse-prod-envoy-1

# Get certificates for both domains
sudo certbot certonly --standalone \
  -d telemetry.yourdomain.com \
  -d otel.yourdomain.com

# Copy certificates
sudo cp /etc/letsencrypt/live/telemetry.yourdomain.com/fullchain.pem /opt/pulse/certs/
sudo cp /etc/letsencrypt/live/telemetry.yourdomain.com/privkey.pem /opt/pulse/certs/

# Restart Envoy
sudo docker start pulse-prod-envoy-1
```

## Alerting

Alertmanager routes alerts via webhook. For local notifications:

```bash
./scripts/start-local-webhook.sh
```

Configure webhook URL in `config/alertmanager.yaml`.

## Troubleshooting

### Check service status
```bash
ssh ec2-user@<IP> "sudo docker ps"
```

### View logs
```bash
ssh ec2-user@<IP> "sudo docker logs pulse-prod-otelcol-1"
ssh ec2-user@<IP> "sudo docker logs pulse-prod-grafana-1"
```

### Restart services
```bash
ssh ec2-user@<IP> "cd /opt/pulse && sudo docker-compose -f docker-compose.prod.yaml restart"
```

### Test OTLP endpoint
```bash
curl -v https://otel.yourdomain.com/v1/traces \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer <your-token>' \
  -d '{}'
```

## Security

- **Change default passwords immediately**
- **Keep `.env` out of version control** (already in `.gitignore`)
- **Use Let's Encrypt for production TLS**
- **Restrict SSH access** to your IP in security group
- **Rotate OTLP tokens** periodically

## License

Open source under the MIT License.
