"""
Example demonstrating automatic .env file loading.

Create a .env file in the project root with:
    SERVICE_NAME=env-demo
    SERVICE_VERSION=2.0.0
    SERVICE_ENVIRONMENT=production
    LOG_LEVEL=WARNING
    MCAP_ENABLED=true
    MCAP_PATH=/tmp/env-demo.mcap
"""

from pulse import Pulse, from_env

def main():
    # Load configuration from environment variables (.env file is auto-loaded)
    service_opts, pulse_opts = from_env()
    
    # Create Pulse instance with env configuration
    pulse = Pulse(service_opts, pulse_opts)
    
    # Test logging - only WARNING and above will show if LOG_LEVEL=WARNING
    pulse.logger.debug("This is a debug message")
    pulse.logger.info("This is an info message")
    pulse.logger.warning("This is a warning message", {
        "loaded_from": "environment",
        "service": service_opts.name,
        "version": service_opts.version,
        "environment": service_opts.environment.value,
    })
    pulse.logger.error("This is an error message")
    
    pulse.close()


if __name__ == "__main__":
    main()
