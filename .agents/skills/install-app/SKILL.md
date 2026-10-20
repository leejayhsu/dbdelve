---
name: install-app
description: Build the release DBDelve.app from this repository and install it into /Applications, replacing any copy already there. Use whenever the user wants to install, update, reinstall or "put in Applications" their own build of DBDelve, or to refresh the installed app after pulling new commits, even if they only say "build the app" and mean the one they launch from the Dock. Not for running a development build (cargo run, or dev/bundle.sh's DBDelve Dev.app).
---

# Install DBDelve into /Applications

`scripts/install.sh` does the whole job: it checks the Xcode prerequisites,
runs `DBDELVE_CHANNEL=release dev/bundle.sh`, and copies the result to
`/Applications/DBDelve.app`, verifying the signature before the old copy is
removed.

## Run it

```bash
.agents/skills/install-app/scripts/install.sh
```

The release build compiles with thin LTO and one codegen unit, so even an
incremental build takes minutes and a clean one far longer. Run it in the
background (or with the maximum timeout) and tell the user it is building
rather than going quiet.

## What each exit code means

- **0**: installed. Report the version it printed. Don't open the app unless
  the user asked: they may be about to open it from the Dock themselves.
- **1**: the build or the copy failed. Show the relevant part of the output.
  A compile error in the build is a code problem in the repository, not an
  install problem.
- **2**: Xcode is not set up for the Metal shader compiler GPUI needs. Pass
  the script's message on. These fixes need the user's password (`sudo`), so
  give them the commands rather than running them:
  1. install Xcode from the App Store
  2. `sudo xcode-select -s /Applications/Xcode.app/Contents/Developer`
  3. `sudo xcodebuild -runFirstLaunch`
  4. `xcodebuild -downloadComponent MetalToolchain`
- **3**: the installed DBDelve is running. Replacing a running app's files
  breaks it, so ask the user to quit it, or ask whether you may quit it for
  them (`osascript -e 'quit app id "com.shayanabbas.dbdelve"'`, which quits
  cleanly so DBDelve saves its open buffers). Then rerun with
  `--install-only`, which skips rebuilding.
- **4**: DBDelve was installed through Homebrew. Overwriting it would leave
  the cask out of step. Ask whether the user wants to uninstall the cask
  (`brew uninstall --cask dbdelve`) before going further; it doesn't touch
  their connections or passwords.

## Worth telling the user

- The release app and the dev app (`DBDelve Dev.app`) keep separate
  connections and Keychain passwords, so connections made in one don't appear
  in the other.
- macOS may ask for Keychain access again after each install. That stops if
  they create the signing certificate once with `dev/identity.sh`.
- No quarantine step is needed: macOS only quarantines downloaded apps, and
  this one was built locally.
