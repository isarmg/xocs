# Browser media fixtures

`image.png` is a 2 × 2 pixel solid-color PNG. `background.webm` contains two
16 × 16 pixel VP8 frames (0.2 seconds), without audio. Both are synthetic test
data, served only by browser fixture routes or uploaded into disposable smoke
instances. They are not included in production assets.

Regenerate with FFmpeg:

```sh
ffmpeg -f lavfi -i color=c=0x6496c8:s=2x2 -frames:v 1 -threads 1 -map_metadata -1 -fflags +bitexact -flags:v +bitexact -y test/browser/fixtures/image.png
ffmpeg -f lavfi -i color=c=0x6496c8:s=16x16:r=10 -frames:v 2 -an -c:v libvpx -deadline best -cpu-used 0 -b:v 20k -map_metadata -1 -fflags +bitexact -flags:v +bitexact -y test/browser/fixtures/background.webm
```
