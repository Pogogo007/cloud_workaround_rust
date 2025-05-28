# cloud config rust workaround
Workaround for games with graphics settings synced by Steam cloud, rewritten in rust.
# All credit for the idea and main logic goes to the Original Creator (tmplshdw)
# [Original](https://github.com/tmplshdw/cloud_config_workaround)

## Why?
I wanted a version that built into native binaries that also supported multiple entries for a single game.  
This is useful for games like Clair Obscur which seem to save their settings in multiple files instead of one.  
I mostly made this for myself but feel free to make an issue if theres something you want added and ill see if i have time for it.  

## Added games
* Clair Obscur - Expedition 33

## Want to add more games?
Follow the same instructions as the original. Except this time you can add multiple entries for one game.  
I recommend making pull requests to the original if your game only has one config that needs to be managed.  
More people will have access to it that way.  
If the game has multiple entries feel free to make a pr here.  

## Usage
Works almost exactly the same as the original  
Download the build corresponding to your os and unpack it somewhere.  
Mark it as executable if needed.  
Add  
`WHEREVER_YOU_PUT_IT/cloud_workaround_rust %command%` for Linux.  
`WHEREVER_YOU_PUT_IT/cloud_workaround_rust.exe %command%` for Windows.  

