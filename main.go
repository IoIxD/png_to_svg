package main

import (
	"image/png"
	"os"
	"log"
	"strings"
	"strconv"
	"fmt"
	"regexp"
	"path/filepath"
)

// Global variables
var svgHeader string 
var svgDefs []string
var svgLinesStripped []string
var svgLines []string
var svgFooter string = "</svg>"
var svgDefNum int = 0
var svgLineNum int = 0
var svgLineStrippedNum int = 0
var w float64 = 1 // the width and height of each pixel placed in the loop
var lx int = 0 // buffer for the x and y we use in the image
var ly int = 0 // buffer for the x and y we use in the image

func main() {
	if(len(os.Args) <= 1) {
		log.Fatalln("You must provide one or more image to convert!")
	} else {
		for _, filen := range os.Args[1:] {
			filena, err := filepath.Glob(filen)
			if(err != nil) {log.Fatalln(err)}
			for _, filenam := range filena {
				fmt.Println(filenam)
				convert(filenam)
			}
		}
	}
}

func initWithSize(w int, h int) { // a function to reinitaelize some of the above globals when we have a variable they need
	svgHeader = "<svg viewBox='0 0 "+s(w)+" "+s(h)+"' xmlns='http://www.w3.org/2000/svg'>"
	// initalize some arrays based on the width/height (plus 2 to account for weird errors)
	size := w*h+2
	svgDefs = make([]string, size)
	svgLinesStripped = make([]string, size)
	svgLines = make([]string, size)
	// reinitalize some variables in case this program is getting more then once
	svgDefNum = 0
	svgLineNum = 0
	svgLineStrippedNum = 0
}

func s(num int) string { // shorthand to return a number as a string
	return strconv.Itoa(num)
}
func sf(num float64) string { // the same idea but for floats
	return fmt.Sprintf("%.1f", num)
}
func h(num int) string { // shorthand to convert a number to it's hex equivalant.
	return fmt.Sprintf("%02x", num)
}
func rgb2hex(r uint32, g uint32, b uint32) string { // self explanatory function
	return "#"+h(int(r/256))+h(int(g/256))+h(int(b/256))
}

// function to create a specific string with specific values
func newBox(w float64, x int, y int, r uint32, g uint32, b uint32) string {
	var val string
	// append the svg with a box with the appropriate values
	val += "<rect width='"+sf(w)+"' height='"+sf(1.1)+"' x='"+s(x)+"' y='"+s(y)+"'"
	val += " fill='"+rgb2hex(r,g,b)+"'>"
	val += "</rect>"
	return val
}

func filter(line string, line_stripped string) string {
	// go through all of the saved lines we have and see if what we're about to add has been added before
	found := false
	// check the defs first
	for i := 0; i < svgDefNum; i++ {
		// if we find one...
		if(svgDefs[i] == "<g id='"+s(i)+"'>"+line_stripped+"</g>") {
			// make the line we have a reference to it instead
			line = "<use href='#"+s(i)+"' x='"+s(lx)+"' y='"+s(ly)+"'></use>"
			found = true
			i = svgDefNum
		}
	}
	// if nothing found...
	if(found == false) {
		// check each of the (stripped) lines we have
		for i := 0; i < svgLineStrippedNum; i++ {
			// if something is found, place it into the defs
			if(svgLinesStripped[i] == line_stripped) {
				svgDefs[svgDefNum] = "<g id='"+s(svgDefNum)+"'>"+line_stripped+"</g>"
				// and make the line we currently have a reference to the new group
				line = "<use href='#"+s(svgDefNum)+"' x='"+s(lx)+"' y='"+s(ly)+"'></use>"
				// and increment the number of definitions
				svgDefNum++
				// and stop the loop
				i = svgLineStrippedNum
			}
		}
	}
	return line
}

func convert(filename string) {
	f, err := os.Open(filename)
	if(err != nil) {log.Fatalln(err)}
	defer f.Close();

	img, err := png.Decode(f);

	// buffer for storing a combined rgb value
	lastR, lastG, lastB, lastA := img.At(-1, -1).RGBA() 

	// the width and height of the file
	fw := img.Bounds().Max.X
	fh := img.Bounds().Max.Y

	// reinit some global variables with this in mind
	initWithSize(fw,fh)

	// regex for later to remove some shit from a string
	re := regexp.MustCompile(`(.*)( x='.*?' y='.*?')(.*)`)

	// for each pixel in the image...
	for y := img.Bounds().Min.Y; y < fh; y++ {
		for x := img.Bounds().Min.X; x < fw; x++ {
			r, g, b, a := img.At(x, y).RGBA() 
			if(a > 1) { // don't even process shit if we're on a transparent pixel
				// if what we have is different from what is stored, make a new box
				if(r+g+b != lastR+lastG+lastB) {
					// don't display black pixels. see the comment at the bottom of the file.
					if(lastR+lastG+lastB > 1) {
						newline := newBox(w+0.2,lx,ly,lastR,lastG,lastB) // create a box
						newnewline := string(re.ReplaceAll([]byte(newline), []byte("$1$3"))) // create a version of that box with no x or y specified
						// filter the line for anything repeatable
						line := filter(newline, newnewline)
						// regardless, add what we have to the saved lines
						svgLines[svgLineNum] = line
						svgLineNum++
						svgLinesStripped[svgLineStrippedNum] = newnewline
						svgLineStrippedNum++
					}
					// reset the positioning and default values for the next pixel.
					w = 1
					lastR, lastG, lastB, lastA = r, g, b, a
					// store the x and y that we'll create the next box from
					lx, ly = x, y 
				} else {
					// increase the side of the box we'll make
					w += 1
				}
			} else {
				// ...unless the last pixel we checked wasn't transparent
				if(lastA > 1) {
					newline := newBox(w+0.2,lx,ly,lastR,lastG,lastB)
					newnewline := string(re.ReplaceAll([]byte(newline), []byte("$1$3")))
					line := filter(newline, newnewline)
					svgLines[svgLineNum] = line
					svgLineNum++
					svgLinesStripped[svgLineStrippedNum] = newnewline
					svgLineStrippedNum++
					w = 1
					lastR, lastG, lastB, lastA = r, g, b, a
					lx, ly = x, y 
				}
			}
		}
	}

	// begin constructing the svg
	svg := svgHeader
	// add the defs
	svg += "<defs>"
	for i := 0; i < svgDefNum; i++ {
		svg += svgDefs[i]
	}
	svg += "</defs>"
	// now, the lines themselves
	for i := 0; i < svgLineNum; i++ {
		svg += svgLines[i]
	}
	// finally, the footer
	svg += svgFooter
	// and save it to a file now
	f2, _ := os.Create(strings.Replace(filename, ".png",".svg", 55))
	f2.Write([]byte(svg))
	f2.Close()
}

// there is an EXTREMELY weird bug where the "if(a > 1)" rings true immediately after we enter a transparency area (even though it's now 0), and I cannot for the life of me find out what it is. maybe one day i will, but it seems the best solution is actually to prevent the program from making any black boxes, and PNGs will need to be modified to use extremely dark greys (254,254,254) instead.