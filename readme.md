# Dotilim
Changing desktop wallpapers, at an interval, on your favorite Linux distribution. The goal of this project is to get a dotfile friendly configuration standard for wallpapers.

To achieve this, the idea is to:  
A) Define a wallpaper configuration specification  
B) Provide a CLI in Rust to use.  
C) Provide a reference implementation library in Rust.  

> Currently the project is mainly focused on B, the CLI, consider the whole project alpha.

## Configuration
Located in `$XDG_CONFIG_HOME/dotilim.toml` or `~/.config/dotilim.toml`, an example:
```toml
version = 1
sources = ["~/Pictures/Wallpapers/**/*.jpg"]
duration = 60
order = "random"
```

### Fields
#### `version`
The configuration version, latest at the moment is: `1`.
#### `sources`
A list of paths to your wallpapers, an item can also use unix shell style globs.
#### `duration`
Number of seconds a wallpaper is shown before changing if multiple wallpapers are given.
#### `order`
Either `Alphabetical` or `Random`.

## Running as a system service
To run Dotilim in the background automatically: Place the following content in either `/etc/systemd/user/dotilim.service` or `~/.config/systemd/user/dotilim.service`. Packagers of systemd-based distributions are encouraged to include the file in the former location.

```toml
[Unit]
Description=dotilim
Documentation=https://github.com/Siilwyn/dotilim

[Service]
ExecStart=/usr/bin/dotilim
Restart=always
RestartSec=12

[Install]
WantedBy=default.target
```

The following example commands will run the service once and enable the service to always run on login in the future respectively:

```
systemctl --user start dotilim.service
systemctl --user enable dotilim.service
```

## Specification
> Version 1
### Configuration types
```
version: Integer
sources: Array<String>
duration: Integer
order: Enum { random, alphabetical }
```

### Field parsing
#### `sources`
Input: Array containing (glob) file paths.  
Valid output: Each path shell resolved and every glob path expanded filling a new array of paths.  
Error output: Array of errors, each error matching an incorrect path.
#### `duration`
Input: Integer of seconds a wallpaper is shown before changing if multiple wallpapers are given.  
Valid output: Integer, same as input.  
Error output: Parsing error containing the given and expected type.
#### `order`
Input: Enum of valid orders.  
Valid output: Order Enum, same as input.  
Error output: Parsing error containing the given and expected type.

___

*What’s in a name?  
The project name Dotilim is a twist on the word 'dotillism' which is the  art of painting in dots. Instead Dotilim works with pixels on your desktop background...*
