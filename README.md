## PNGToSVG

A program for converting PNGs to SVGs without any tracing support or the like. A rectangle element is made for each unique set of pixels, and commonly used pixels are stored in defs to save on file size. If you are looking to convert a PNG to an SVG without sharp edges, you should be [here](https://www.pngtosvg.com/) instead.

This is an inheriently bad solution due to the file size (3 times bigger then a PNG) but if you are looking to make a modern website that the general public might see, with pixel perfect graphics, this is basically your only option, because on displays higher then 1080p (which are becoming more common) these kinds of images scale weirdly. It's slight, but it's enough to look awful, and this is where this solution comes in handy.

Demonstration on a 2048x1536 display:

![demonstartion](https://ioi-xd.net/files/demonstration.png)

...oh yeah, except for the fact that it's still technically antialiased, and shapeRendering it actually makes it look a bit worse then turning anti aliasing off on a png. Because of the nature of SVG, though, it is *less* blurry, and on higher screens the blur is barely noticable to most people (unless they take a screenshot of the screen and zoom in). You might consider a solution where you display pngs at lower resolutions, but this is actually not necessary because the anti aliasing only happens on screens above 1080p. Sadly, this is obviously not a perfect solution, but with the advent of higher res displays, it's certainly the best way forward.
  
# Compiling/Usage
Requires go 1.17. It exclusively uses libraries built into golang, so in theory, you should be able to do `go build main.go` and get an executable for your operating system.

As you might guess by that last paragraph, it's a terminal application. The only arguments it takes are images to convert, and wildcards are supported so you can convert an entire folder of pngs.

# Known limitations
* Perfect blacks don't get rendered. This is not a bug, but a solution to a bug. I should eventually fix this, but in the mean time you will need to modify your images to use very dark grays instead
* Only PNGs are supported. GIFs (non-animated) and JPEGs will be supported later.
* Transparent images work best, there is an inconsistent bug where if the rightmost column of pixels isn't transparent then the resulting image is corrupted