# Deployment instructions

- Install [Pi OS Lite](https://www.raspberrypi.com/software/operating-systems/) (64 bit)
  - Install updates
  - Set hostname
  - Enable SSH and add keys
- (optional) Install and configure Tailscale
- Build the [controller](../controller): `cross build --release`
- [Deploy via Ansible](../controller-deploy): `ansible-playbook controller.yml`
