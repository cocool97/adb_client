# adb_remote_auth

A custom `adb` implementation based on [`adb_client`](https://github.com/cocool97/adb_client) to provide remote authentication support through a custom API. See [`adb_signer`](https://github.com/openaipin/adb_signer) for the compatible server implementation.

The intended usage of this variant of ADB is to allow access to locked down Android devices where the certificates cannot be safely distributed. To connect to the Humane Ai Pin using the openPin signing server, you can add `--remote-auth-url https://adb.openpinsigning.workers.dev` to your commands:

```bash
adb_remote_auth usb --remote-auth-url https://adb.openpinsigning.workers.dev shell ls
```


## Usage

Usage is quite simple, and tends to look like `adb`:

> [!WARNING]  
> macOS users probably need to run `adb_remote_auth` with `sudo` due to permission issues.

```bash
$ adb_remote_auth usb --help
Device commands via USB, no server needed

Usage: adb_remote_auth usb [OPTIONS] --vendor-id <VID> --product-id <PID> <COMMAND>

Commands:
  shell   Spawn an interactive shell or run a list of commands on the device
  pull    Pull a file from device
  push    Push a file on device
  stat    Stat a file on device
  reboot  Reboot the device
  help    Print this message or the help of the given subcommand(s)

Options:
  -a, --remote-auth-url <URL>              URL for remote ADB authentication
  -v, --vendor-id <VID>                    Hexadecimal vendor id of this USB device
  -p, --product-id <PID>                   Hexadecimal product id of this USB device
  -k, --private-key <PATH_TO_PRIVATE_KEY>  Path to a custom private key to use for authentication
  -h, --help                               Print help
```

Append `--remote-auth-url` to the middle of your command string to bypass local certificates and use a remote signing server. For the Humane Ai Pin, add `--remote-auth-url https://adb.openpinsigning.workers.dev` to your commands:

```bash
adb_remote_auth usb --remote-auth-url https://adb.openpinsigning.workers.dev shell ls
```
