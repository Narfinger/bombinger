# Bombinger
This downloads giant bomb videos. This is mostly used with plex or similar software. The file bombinger.service and bombinger.time can be used with systemd.
Change the bombinger.service ExecStart to show the complete path of the compiled version. Symlink the .service and .timer file into ~/.config/systemd/user/ Then enable it with systemctl --user enable bombinger.timer.

It might be useful to start it with systemctl start --user bombinger.service once. This will create the config.toml file (most likely in your home directory). Add path and your gbkey and you should be up and running.