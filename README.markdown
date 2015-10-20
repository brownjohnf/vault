# Vault

Vault is a simple bash script to automate the mounting of encrypted volumes.

# Install

Put `bin/vault` anywhere in your path.

# Usage

In all examples, mountpoint should be the *relative* path for the mountpoint.
All mountpoints will be prefixed by `/mnt`.

Create a new vault:

```bash
vault create /dev/sdz1 mountpoint
```

Mount an existing vault:

```bash
vault mount /dev/sdz1 mountpount
```

Unmount a mounted vault:

```bash
vault umount mountpoint
```

