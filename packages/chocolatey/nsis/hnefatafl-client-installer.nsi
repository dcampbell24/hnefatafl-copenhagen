;Hnefatafl Copenhagen
;Written by David Lawrence Campbell

;--------------------------------
;Include Modern UI

  !include "MUI2.nsh"

;--------------------------------
;General

  ;Name and file
  Name "Hnefatafl"
  OutFile "..\hnefatafl-copenhagen\tools\hnefatafl-client-installer-1.1.3.exe"
  Unicode True

  ;Default installation folder
  InstallDir "$LOCALAPPDATA\hnefatafl-copenhagen"

  ;Get installation folder from registry if available
  InstallDirRegKey HKCU "Software\hnefatafl-copenhagen" ""

  ;Request application privileges for Windows Vista
  RequestExecutionLevel user

;--------------------------------
;Interface Settings

  !define MUI_ABORTWARNING

;--------------------------------
;Pages

  !insertmacro MUI_PAGE_LICENSE "..\..\..\LICENSE-MIT"
  !insertmacro MUI_PAGE_COMPONENTS
  !insertmacro MUI_PAGE_DIRECTORY
  !insertmacro MUI_PAGE_INSTFILES

  !insertmacro MUI_UNPAGE_CONFIRM
  !insertmacro MUI_UNPAGE_INSTFILES

;--------------------------------
;Languages

  !insertmacro MUI_LANGUAGE "English"

;--------------------------------
;Installer Sections

Section "Hnefatafl" SecHnefatafl

  SetOutPath "$INSTDIR"

  File "..\..\..\target\release\hnefatafl-client.exe"
  File "king_256x256.ico"

  ;Store installation folder
  WriteRegStr HKCU "Software\hnefatafl-copenhagen" "" $INSTDIR

  ;Create uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  ;Create Start Menu entry
	CreateShortcut $SMPROGRAMS\Hnefatafl.lnk $INSTDIR\hnefatafl-client.exe "" "$INSTDIR\king_256x256.ico" 0

SectionEnd

;--------------------------------
;Descriptions

  ;Language strings
  LangString DESC_SecHnefatafl ${LANG_ENGLISH} "Hnefatafl Copenhagen client that connects to a server."

  ;Assign language strings to sections
  !insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SecHnefatafl} $(DESC_SecHnefatafl)
  !insertmacro MUI_FUNCTION_DESCRIPTION_END

;--------------------------------
;Uninstaller Section

Section "Uninstall"

  Delete "$INSTDIR\hnefatafl-client.exe"
  Delete "$INSTDIR\Uninstall.exe"
  Delete "$SMPROGRAMS\Hnefatafl.lnk" 

  RMDir "$INSTDIR"

  DeleteRegKey /ifempty HKCU "Software\hnefatafl-copenhagen"

SectionEnd
