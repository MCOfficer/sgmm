# The Stellaris GOG Mod Manager (`sgmm`)

A helper program that installs mods from the steam workshop to your GOG Stellaris. Currently supports on Linux - Windows support should be straightfoward, but difficult (for me) to test.

## Usage

```
❯ ./sgmm install https://steamcommunity.com/sharedfiles/filedetails/\?id\=2077186491
Installing mod 2077186491

Fetching info

### Downloading ###                                                                                                                  
Attempting download via steamworkshop.download
Failed to request download: Resource temporarily unavailable (os error 11)
Attempting download via steamworkshopdownloader.io
Downloading https://api.steamworkshopdownloader.io/api/download/transmit?uuid=d7a52064-d712-4679-9381-545f229f8f2f

### Installing ###
Extracting to /home/florian/.local/share/Paradox Interactive/Stellaris/mod/steam_2077186491
Writing .mod file
Removing mods_registry.json
Open the Launcher once to regenerate it; until then, Stellaris won't recognize your mods.

Done!

❯ ./sgmm remove 2077186491 # Also accepts IDs
Removing mod 2077186491

Done!
```

## Installing
Prebuilt binaries are available on the [releases page](https://github.com/MCOfficer/sgmm/releases). The CI runs also create binaries, in case you like the bleeding edge.

