Add-Type -AssemblyName System.Drawing

$sourcePath = Join-Path $PSScriptRoot '..\assets\private\chun_li_source_r4.png'
$outputPath = Join-Path $PSScriptRoot '..\assets\private\chun_li_r4.png'

$source = [System.Drawing.Bitmap]::new($sourcePath)
$output = [System.Drawing.Bitmap]::new(
    $source.Width,
    $source.Height,
    [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
)

try {
    $graphics = [System.Drawing.Graphics]::FromImage($output)
    try {
        $graphics.DrawImageUnscaled($source, 0, 0)
    }
    finally {
        $graphics.Dispose()
    }

    $rect = [System.Drawing.Rectangle]::new(0, 0, $output.Width, $output.Height)
    $data = $output.LockBits(
        $rect,
        [System.Drawing.Imaging.ImageLockMode]::ReadWrite,
        [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
    )
    try {
        $byteCount = [Math]::Abs($data.Stride) * $output.Height
        $pixels = [byte[]]::new($byteCount)
        [System.Runtime.InteropServices.Marshal]::Copy($data.Scan0, $pixels, 0, $byteCount)

        for ($i = 0; $i -lt $pixels.Length; $i += 4) {
            $blue = $pixels[$i]
            $green = $pixels[$i + 1]
            $red = $pixels[$i + 2]
            if ($red -ge 248 -and $green -le 8 -and $blue -ge 248) {
                $pixels[$i + 3] = 0
            }
        }

        [System.Runtime.InteropServices.Marshal]::Copy($pixels, 0, $data.Scan0, $byteCount)
    }
    finally {
        $output.UnlockBits($data)
    }

    $output.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Png)
}
finally {
    $output.Dispose()
    $source.Dispose()
}

Write-Host "Processed Chun-Li R4 sheet: $outputPath"
