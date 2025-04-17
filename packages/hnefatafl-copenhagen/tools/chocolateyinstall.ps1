$ErrorActionPreference = 'Stop'
$toolsDir     = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$fileLocation = Join-Path $toolsDir 'hnefatafl-client-installer.exe'

$packageArgs = @{
  packageName   = $env:ChocolateyPackageName
  unzipLocation = $toolsDir
  fileType      = 'EXE'
  file          = $fileLocation

  softwareName  = 'Hnefatafl'

  # You can use checksum.exe (choco install checksum)
  # checksum -t sha256 -f hnefatafl-client-installer.exe

  checksum64     = '40D253173977199510F34820F4D4C47B078EDA49FCF9829FEE354A1734524680'
  checksumType64 = 'sha256'

  # NSIS
  silentArgs     = '/S'
  validExitCodes = @(0)
}

Install-ChocolateyInstallPackage @packageArgs
