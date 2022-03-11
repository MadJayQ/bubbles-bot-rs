@echo off
pushd %~dp0\
cls
echo Working directory %cd%
set "DISCORD_TOKEN=ODg5OTcyOTU3MTk0MDk2Njkw.YUpB5w.OebGUIXq7XKlMrF2c60pan6r1Ss"
popd
cargo run -- -i="%assetdir%" -o="%outputdir%" -n="Test"