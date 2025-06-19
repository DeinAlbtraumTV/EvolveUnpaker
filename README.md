# Evolve Unpaker

This is a tool for unpacking all .pak files from Evolve. The actual pak decrypter is NOT included, see instructions below on how to use this!

## How to use
- clone this repo
- make sure to clone the zip2 submodule, as i had to patch out the CRC verification (for some reason some files have bad CRCs but still work...)
- build the program: `cargo build --release`
- Copy the EvolveUnpaker.exe as well as a pak decrypter for Evolve into a folder next to your EvolveGame folder
  - If your EvolveGame folder is located in .../steamapps/common/EvolveGame, it should look something like this (EvolveTools can be named differently):
    - .../steamapps/common/EvolveTools/EvolveUnpaker.exe
- start a geckodriver instance on port 4444
- run EvolveUnpaker.exe
- make sure to position your mouse directly over the code area of the decompiler and DO NOT move it while the tool is running, as mouse inputs will be simulated to extract the decompiled lua code
- to also copy all non-pak files, run `EvolveUnpaker.exe --copy-non-paks` after the first run has finished

## How it works
- the Unpaker goes through all pak files of the original game
- it starts by copying the currently processed pak file into the EvolveTools folder
- the file then gets decrypted using PakDecrypt.exe
- the resulting zip file then gets unpacked into a new Folder called EvolveUnpacked, in the same parent dir as the EvolveGame and EvolveTools folders
- all lua files in the unpacked zip get decompiled using https://luadec.metaworm.site/, a decompiled version of the lua file will be stored as a .decomp.lua file next to the original

## Usage warning
This tool relies on simulation keystrokes and mouse inputs to interact with the decompiler website. I highly recommend to let this run over night, as moving the mouse/keyboard while the unpaker is running will most likely result in garbage decompiled lua code.\
Also, make sure the version of PakDecrypt you are using has the correct key for the Evolve version you are trying to unpack.

## Links
- Pre-Unpaked modded Evolve files (Patch Version 1.4): https://evolve.a1btraum.de/modded/unpack/
- PakDecrypt for modded Evolve: https://evolve.a1btraum.de/modded/PakDecrypter.zip
- PakDecrypt for vanilla Evolve: https://evolve.a1btraum.de/PakDecrypter.zip
- Pre-Packed version of the EvolveTools folder, including geckodriver and modded PakDecrypt: https://evolve.a1btraum.de/modded/unpack/EvolveTools.zip