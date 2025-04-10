$ErrorActionPreference = 'Stop'
$toolsDir   = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$fileLocation = Join-Path $toolsDir 'hnefatafl-client-installer.exe'

$packageArgs = @{
  packageName   = $env:ChocolateyPackageName
  unzipLocation = $toolsDir
  fileType      = 'EXE'
  file          = $fileLocation

  softwareName  = 'Hnefatafl'

  # You can use checksum.exe (choco install checksum)
  # checksum -t sha256 -f path\to\file
  checksum      = '7673B5702855707794A13EE8085322F10A8F167291BF3055A0D877313766C4DC'
  checksumType  = 'sha256'
  checksum64    = '7673B5702855707794A13EE8085322F10A8F167291BF3055A0D877313766C4DC'
  checksumType64= 'sha256'

  # NSIS
  silentArgs   = '/S'
  validExitCodes= @(0)
}

Install-ChocolateyInstallPackage @packageArgs

$osBitness = Get-ProcessorBits
