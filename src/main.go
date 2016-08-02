package main

import (
	"fmt"

	"GBVideos"

	"github.com/jroimartin/gocui"
)

func main() {
	var m = GBVideos.GetShows()
	fmt.Println(m)
	// g := gocui.NewGui()
	// if err := g.Init(); err != nil {
	// 	log.Panicln(err)
	// }
	// defer g.Close()

	// g.SetLayout(layout)

	// if err := g.SetKeybinding("", gocui.KeyCtrlC, gocui.ModNone, quit); err != nil {
	// 	log.Panicln(err)
	// }

	// if err := g.MainLoop(); err != nil && err != gocui.ErrQuit {
	// 	log.Panicln(err)
	// }
}

func layout(g *gocui.Gui) error {
	maxX, maxY := g.Size()
	if v, err := g.SetView("hello", maxX/2-7, maxY/2, maxX/2+7, maxY/2+2); err != nil {
		if err != gocui.ErrUnknownView {
			return err
		}
		fmt.Fprintln(v, "Hello world!")
	}
	return nil
}

func quit(g *gocui.Gui, v *gocui.View) error {
	return gocui.ErrQuit
}