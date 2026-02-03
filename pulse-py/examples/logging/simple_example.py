"""
Simple logging example using the new Pulse.new() builder API.

Configuration is auto-discovered from:
1. pulse.toml in current directory
2. Environment variables (PULSE_*)
3. Builder method overrides (highest priority)

Run with:
    uv run python -m examples.logging.simple_example
"""

from pulse import Pulse


def main():
    # Auto-discovers pulse.toml config file
    # No builder overrides - uses config file values
    with Pulse.new().build() as pulse:
        
        pulse.logger.info("Chat service started")
        pulse.logger.debug("Debug mode enabled")
        pulse.logger.info("OpenTelemetry logging example")
        
        active_rooms = 3
        total_users = 42
        pulse.logger.info(f"Service initialized with {active_rooms} active rooms and {total_users} users")
        
        pulse.logger.warning("Rate limit approaching", {
            "current_percent": 85.5,
            "user_id": "user-123",
        })
        
        pulse.logger.error("Failed to process message", {
            "error_code": 500,
            "user_id": "user-456",
        })
        
        pulse.logger.info("Chat service shutting down")


if __name__ == "__main__":
    main()
