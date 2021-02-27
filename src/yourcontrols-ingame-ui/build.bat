@echo off
cd ui-src
cmd /c "npm run build"
cd ..
RMDIR /Q/S "ingame-panel\html_ui\InGamePanels\CustomPanel\UI"
mkdir "ingame-panel\html_ui\InGamePanels\CustomPanel\UI"
xcopy /s/e/d "ui-src\dist" "ingame-panel\html_ui\InGamePanels\CustomPanel\UI"
"%MSFS_SDK%Tools\bin\fspackagetool.exe" "ingame-panel\Build\maximus-ingamepanels-custom.xml" -nomirroring
copy /Y "ingame-panel\Build\Packages\maximus-ingamepanels-custom\Build\maximus-ingamepanels-custom.spb" "ingame-panel\InGamePanels"
cd ingame-panel
py "..\build.py"
pause