{ config, lib, pkgs, modulesPath, ... }:

{
  imports = [
    "${modulesPath}/virtualisation/amazon-image.nix"
    ../modules/opentelementry-telemetry.nix
  ];

  # EC2 specific settings
  ec2.hvm = true;

  # System settings
  system.stateVersion = "24.05";

  # Enable Opentelementry Telemetry
  services.opentelementry-telemetry = {
    enable = true;
    # These will be overridden by deployment
    domain = "telemetry.example.com";
    otelDomain = "otel.example.com";
    acmeEmail = null; # Set to enable Let's Encrypt
  };

  # Basic system packages
  environment.systemPackages = with pkgs; [
    vim
    htop
    curl
    jq
    openssl
  ];

  # SSH access
  services.openssh = {
    enable = true;
    settings = {
      PasswordAuthentication = false;
      PermitRootLogin = "prohibit-password";
    };
  };

  # Automatic updates
  system.autoUpgrade = {
    enable = true;
    allowReboot = false;
  };

  # Garbage collection
  nix.gc = {
    automatic = true;
    dates = "weekly";
    options = "--delete-older-than 30d";
  };

  # Enable flakes
  nix.settings.experimental-features = [ "nix-command" "flakes" ];

  # Users
  users.users.opentelementry = {
    isNormalUser = true;
    extraGroups = [ "wheel" "docker" "podman" ];
    openssh.authorizedKeys.keys = [
      # Add your SSH public key here or via deployment
    ];
  };

  # Allow passwordless sudo for opentelementry user
  security.sudo.wheelNeedsPassword = false;
}
