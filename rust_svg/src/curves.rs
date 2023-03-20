/*
A few example space filling curves and plants encoded by
Lindenmayer Systems.
*/

pub let Curves = String::from(r#"

{
  "title" = "Hilbert Curve",
  "refs" = [
    "https://www.cs.unh.edu/~charpov/programming-lsystems.html",
  ],
  "angle" = 90.0,
  "order" = [1,2,3,4],
  "start" = "X",
  "rules" = {
    X = "-YF+XFX+FY-",
    Y = "+XF-YFY-FX+"
  }
}

{
  "title" = "Koch's Snowflake",
  "refs" = [
    "https://www.cs.unh.edu/~charpov/programming-lsystems.html",
  ],
  "angle" = 60.0,
  "order" = [1,2,3,4],
  "start" = "+F--F--F",
  "rules" = {
      F = "F+F--F+F"
  }
)

{
  "title" = "Peano Curve aka Hilbert II",
  "refs" = [
    "http://bl.ocks.org/nitaku/8949471",
    "http://mathworld.wolfram.com/HilbertCurve.html",
  ],
  "angle" = 90.0,
  "order" = [1,2,3,4],
  "start" = "L",
  "rules" = {
    "L" = "LFRFL-F-RFLFR+F+LFRFL",
    "R" = "RFLFR+F+LFRFL-F-RFLFR",
  }
}

  Gosper = LSys(
    "title" = "Peano-Gosper Curve aka 'Flowsnake'",
    "refs" = [
      "https://en.wikipedia.org/wiki/Gosper_curve",
      "http://larryriddle.agnesscott.org/ifs/ksnow/flowsnake.htm",
    ],
    "rules" = dict(
      "angle" = 60.0,
      "order" = [1,2,3,4],
      "start" = "A",
      A = "A-B--B+A++AA+B-",
      B = "+A-BB--B-A++A+B",
    ),
    Post"rules" = dict(
      A = "F",
      B = "F",
    ),
  ),

#  QGosper = LSys(
#    "title" = "Quadratic Gosper",
#    "refs" = [
#      "http://paulbourke.net/fractals/lsys/"
#    ],
#    "rules" = dict(
#      "angle" = 90.0,
#      "order" = [1,2,3,4],
#      "start" = "YF",
#      X = "XFX-YF-YF+FX+FX-YF-YFFX+YF+FXFXYF-FX+YF+FXFX+YF-FXYF-YF-FX+FX+YFYF-",
#      Y = "+FXFX-YF-YF+FX+FXYF+FX-YFYF-FX-YF+FXYFYF-FX-YFFX+FX+YF-YF-FX+FX+YFY",
#    ),
#  ),

  SierpD = LSys(
    "title" = "Sierpinski Diamond",
    "refs" = [
      "http://paulbourke.net/fractals/lsys/",
    ],
    "rules" = dict(
      "angle" = 90.0,
      "order" = [2,3,4,5],
      "start" = "F+XF+F+XF",
      X = "XF-F+F-XF+F+XF-F+F-X",
    ),
  ),

  SierpA = LSys(
    "title" = "Sierpinski Arrowhead",
    "refs" = [
      "http://paulbourke.net/fractals/lsys/",
    ],
    "rules" = dict(
      "angle" = 60.0,
      "order" = [2,3,4,8],
      "start" = "YF",
      X = "YF+XF+Y",
      Y = "XF-YF-X",
    ),
  ),

  SierpSS = LSys(
    "title" = "Sierpinski Square Snowflake",
    "refs" = [
      "http://www.ethoberon.ethz.ch/WirthPubl/AD.pdf#page93",
      "https://en.wikipedia.org/wiki/Sierpi%C5%84ski_curve",
      "http://mathworld.wolfram.com/SierpinskiCurve.html",
    ],
    "rules" = dict(
      "angle" = 45.0,
      "order" = [1,2,3,4],
      "start" = "+BABA",
      A = "F--F--",
      B = "BF+FF+B F--F-- BF+FF+B",
    ),
  ),

  Pent1 = LSys(
    "title" = "Pentaplexity",
    "refs" = ["http://paulbourke.net/fractals/lsys/",
    ],
    "rules" = dict(
      "angle" = 36.0,
      "order" = [1,2,3,4],
      "start" = "F++F++F++F++F",
      F = "F++F++F|F-F++F",
    ),
  ),

  Dragon = LSys(
    "title" = "Dragon Curve",
    "refs" = ["http://paulbourke.net/fractals/lsys/",
    ],
    "rules" = dict(
      "angle" = 90.0,
      "order" = [2,4,6,14],
      "start" = "+FX",
      X = "X+YF+",
      Y = "-FX-Y",
    ),
  ),

  Plant1 = LSys(
    "title" = "Plant 1",
    "refs" = [
      "https://www.cs.unh.edu/~charpov/programming-lsystems.html",
    ],
    "rules" = dict(
      "angle" = 22.5,
      "start" = "++++X",
      X = "F+[[X]-X]-F[-FX]+X",
      F = "FF",
    ),
  ),

  Plant2 = LSys(
    "title" = "Plant 2",
    "refs" = [
      "https://www.cs.unh.edu/~charpov/programming-lsystems.html",
    ],
    "rules" = dict(
      "angle" = 22.5,
      "start" = "++++F",
      F = "FF-[-F+F+F]+[+F-F-F]",
    ),
  ),

)
"#)
