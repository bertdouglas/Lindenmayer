use std::collections::HashMap;
use std::f64::consts::PI;
use std::f64;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

use serde::{Deserialize, Serialize};
use indoc::indoc;

pub mod test_main;

/*----------------------------------------------------------------------
Lindenmayer System interpreter and display using SVG.

An LSys is a set of rules for string substitution. There is a starting
string and a set of rule strings.  Each character in a string is either the
name of another rule, or a special action character.

The special action characters are:
F Move forward by line length drawing a line
f Move forward by line length without drawing a line
+ Turn left by turning angle
- Turn right by turning angle
| Reverse direction (ie: turn by 180 degrees)
[ Push current drawing state onto stack
] Pop current drawing state from the stack

The drawing state consists of:
- drawing direction
- drawing position

Structured Vector Graphics (SVG) is generated to draw the LSys.
This is a rewrite of previous version from python/postscript.
*/

/*----------------------------------------------------------------------
Tune-able parameters
*/

static PARMS : [(&str, &str); 3 ] = [
  ("linewidth",   "0.02"      ),   // inches
  ("pagewidth",   "8.5"       ),   // inches
  ("pageheight", "11.0"       ),   // inches
  //titlefont = "/Times-Bold",
  //titlesize = 30,
  //attrfont = "/Arial",
  //attrsize = 12,
];

fn pget(key:&str) -> &str {
    let mut val:&str = "";
    for (k,v) in PARMS {
        if k == key {
            val = v;
            break;
        }
    }
    val
}

/*----------------------------------------------------------------------
SVG output document state management

This collects page fragments and inserts
document and page wrappers.
*/

// document actions
enum DocAct<'a> {
    // start new document, specify path to output file and comment
    DocOpenPathComment(&'a str, &'a str),
    // Start a new page, and specify comment
    PageStartComment(&'a str),
    // Add a data fragment to the page (content of data is not checked)
    PageAddFragment(&'a str),
    // Close out page and write to file
    PageEnd,
    // Close out document and write to file
    DocClose
}

struct DocState {
    indoc   : bool,             // inside a document
    inpage  : bool,             // inside a page
    page_no : usize,            // number of current page
    frag_no : usize,            // number of current fragment in page
    buf     : Vec<u8>,          // svg output buffer
    file    : Option<File>,     // file in which to write output
}

fn doc_new() -> DocState {
    let ds = DocState {
        indoc   : false,
        inpage  : false,
        page_no : 0,
        frag_no : 0,
        buf     : vec!(),
        file    : None,
    };
    ds
}

fn doc(ds:& mut DocState, doc_act:DocAct) {
    match doc_act {
        DocAct::DocOpenPathComment(path,comment) => {
            // must be completely blank
            assert!(ds.indoc == false);
            assert!(ds.inpage == false);
            assert!(ds.page_no == 0);
            assert!(ds.frag_no == 0);
            assert!(ds.buf.len() == 0);
            assert!(ds.file.is_none());
            // open the file
            ds.file = Some(
                        OpenOptions::new()
                            .write(true).create(true)
                            .open(path).unwrap()
                    );
            // document header
            let svg_doc_head = format!( indoc! {r#"
                <!-- {path}
                     {comment} -->
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    version="1.2"
                    width="{pagewidth}in"
                    height="{pageheight}in"
                >
                <pageSet>

                "#},
                path = path,
                comment = comment,
                pagewidth   = pget("pagewidth"),
                pageheight  = pget("pageheight"),
            );
            ds.buf.append(&mut svg_doc_head.into_bytes());
            // new state
            ds.indoc = true;
        }
        DocAct::PageStartComment(comment) => {
            // assert state
            assert!(ds.indoc == true);
            assert!(ds.inpage == false);
            // emit page header
            ds.page_no += 1;
            ds.frag_no = 0;
            let svg_page_head = format!( indoc! {r#"
                <page>
                <!-- begin page {page_no}
                     {comment} -->
                "#},
                page_no = ds.page_no,
                comment = comment,
            );
            ds.buf.append(&mut svg_page_head.into_bytes());
            // new state
            ds.inpage = true;
        }
        DocAct::PageAddFragment(frag) => {
            // assert state
            assert!(ds.inpage == true);
            // fragment header
            ds.frag_no += 1;
            let svg_frag_head = format!( indoc! {r#"

                <!-- page {page_no} fragment {frag_no} -->
                "#},
                page_no = ds.page_no,
                frag_no = ds.frag_no,
            );
            ds.buf.append(& mut svg_frag_head.into_bytes());
            // collect fragment
            let mut frag:Vec<u8> = frag.as_bytes().to_vec();
            ds.buf.append(&mut frag);
        }
        DocAct::PageEnd => {
            // assert state
            assert!(ds.indoc == true);
            assert!(ds.inpage == true);
            assert!(ds.frag_no > 0);
            assert!(ds.file.is_some());
            let mut file = ds.file.as_mut().unwrap();
            // page footer
            ds.inpage = false;
            let svg_page_foot = format!( indoc! {r#"

                </page>
                <!-- end page {page_no} -->

                "#},
                page_no = ds.page_no,
            );
            ds.buf.append(&mut svg_page_foot.into_bytes());
            // write page
            file.write_all(&ds.buf).ok();
            ds.buf.clear();
        }
        DocAct::DocClose => {
            // assert state
            assert!(ds.indoc == true);
            assert!(ds.inpage == false);
            assert!(ds.page_no > 0);
            assert!(ds.buf.len() == 0);
            assert!(ds.file.is_some());
            let mut file = ds.file.as_mut().unwrap();
            // doc footer
            let svg_doc_foot = format!( indoc! {r#"
                </pageSet>
                </svg>
                "#}
            );
            ds.buf.append(&mut svg_doc_foot.into_bytes());
            // write and close file
            file.write_all(&ds.buf).ok();
            file.sync_all().ok();
            std::mem::drop(file);
            // set new state
            ds.indoc = false;
            ds.buf.clear();
            ds.file = None;
        }
    }
}

/*----------------------------------------------------------------------
Page layout bounding boxes

For convenience of page layout, define bounding boxes for
the regions, "top", "a", "b", "left", "center", "right", "main",
as diagrammed below.

Note that the page origin for SVG is at top left.  This is different
from that used by postscript which is at the bottom left.
The orientation of y-axis for SVG is inverted

    +-----------------0-----------------+
    |                top                |
    +------------1---+------------------+
    |                |                  |
    |       a        2        b         |
    |                |                  |
    +----------+-2---+-----+------------+
    |          |           |            |
    0 left     1  center   3   right    4
    |          |           |            |
    +----------+-----3-----+------------+
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    |               main                |
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    +---------------4-------------------+
*/
type BBox = (f64,f64,f64,f64);
type LayoutBoxes<'a> = HashMap<&'a str,BBox>;
fn layout_boxes_make() -> LayoutBoxes<'static> {

    // all box edges as fraction of page size
    //             0     1     2     3     4
    let xf = vec![0.05, 0.35, 0.50, 0.65, 0.95];
    let yf = vec![0.03, 0.14, 0.20, 0.42, 0.97];

    // scale to page size
    let width:f64  = pget("pagewidth").parse().unwrap();
    let x:Vec<f64> = xf.into_iter().map(|x| x * width).collect();
    let height:f64 = pget("pageheight").parse().unwrap();
    let y:Vec<f64> = yf.into_iter().map(|y| y * height).collect();

    // make named bounding boxes
    HashMap::from([
        (  "main"   , (x[0],y[3],x[4],y[4]) ),
        (  "left"   , (x[0],y[2],x[1],y[3]) ),
        (  "center" , (x[1],y[2],x[3],y[3]) ),
        (  "right"  , (x[3],y[2],x[4],y[3]) ),
        (  "a"      , (x[0],y[1],x[2],y[2]) ),
        (  "b"      , (x[2],y[1],x[4],y[2]) ),
        (  "top"    , (x[0],y[0],x[4],y[1]) ),
    ])
}
fn layout_boxes_draw(boxes: &LayoutBoxes) -> String {
    let mut svg = String::new();

    // foreach box
    for (_k,v) in boxes {
        let s = format!( indoc! {r#"
            <rect
                x      = "{x0:.4}in"
                y      = "{y0:.4}in"
                rx     = "0.1in"
                ry     = "0.1in"
                width  = "{w:.4}in"
                height = "{h:.4}in"
                style  = "
                    fill           :  none;
                    stroke         :  black;
                    stroke-width   :  {strokewidth:.4}in;
                "
            />
            "#},
            x0=v.0,y0=v.1,w=v.2-v.0,h=v.3-v.1,
            strokewidth = pget("linewidth"),
        );
        svg.push_str(&s);
    }
    svg
}


/*----------------------------------------------------------------------
Convert fully elaborated LSys rules into a list of drawing actions.
The drawing actions are in an abstract space with initial position
at (x,y)=(0,0) and all actions having relative motion of one unit
wrt current position.
*/

enum DAct {
    RmoveTo(f64,f64),
    RlineTo(f64,f64)
}

fn lsys_dacts_from_rules(lsys:&LSys, rules:&str) -> (Vec<DAct>,BBox) {
    type DXY = (f64,f64,f64);
    let mut stack:Vec<DXY> = vec!();
    let mut dacts:Vec<DAct> = vec!();

    // direction and angle step
    let mut d:f64 = 0.0;
    let angle:f64 = lsys.angle * PI / 180.0;

    // current position and bounding box
    let (mut x, mut y, mut x0, mut y0, mut x1, mut y1 )
      : (f64,   f64,   f64,    f64,    f64,    f64,   )
      = (0.0,   0.0,   0.0,    0.0,    0.0,    0.0,   );
    let (mut xt, mut yt) : (f64,f64);

    // starting position
    dacts.push(DAct::RmoveTo(0.0,0.0));

    // do the actions
    for rule in rules.chars() {
        // forward
        if 'F' == rule {
            xt = d.cos();       yt = d.sin();
            x += xt;            y += yt;
            dacts.push(DAct::RlineTo(xt,yt));
        }
        else if '+' == rule {
            d += angle;
        }
        else if '-' == rule {
            d -= angle;
        }
        else if '[' == rule {
            stack.push((d,x,y));
        }
        else if ']' == rule {
            (d,xt,yt) = stack.pop().unwrap();
            dacts.push(DAct::RmoveTo(xt-x,yt-y));
            x = xt;  y = yt;
        }
        else if '|' == rule {
            d += PI;
        }
        else {
            panic!("Unimplemented action: '{rule}'");
        }
        // maintain bounding box
        x0 = f64::min(x0,x);     y0 = f64::min(y0,y);
        x1 = f64::max(x1,x);     y1 = f64::max(y1,y);
    }

    // adjust bounding box so it can't have zero size
    if x0==x1 { x0 = -0.1;  x1 = 0.1; }
    if y0==y1 { y0 = -0.1;  y1 = 0.1; }

    (dacts,(x0,y0,x1,y1))
}

/*----------------------------------------------------------------------
Produce svg to draw LSys at specified order to fit in specified
layout box on page.  Units are inches.
svg output is a string.
*/

fn lsys_draw_basic(lsys:&LSys, order:i32, pbb:BBox) -> String {
    let mut svg = String::new();
    let (px0,py0,px1,py1) = pbb;
    let rules = lsys_apply_rules(lsys,order);
    let (dacts,abb) = lsys_dacts_from_rules(lsys,&rules);
    let (ax0,ay0,ax1,ay1) = abb;

    // find scale factor
    let dp = f64::sqrt((px1-px0)*(py1-py0));
    let da = f64::sqrt((ax1-ax0)*(ay1-ay0));
    let scale = dp/da;

    // find starting position
    let x0 = (px0+px1)/2.0 - scale*(ax0+ax1)/2.0;
    let y0 = (py0+py1)/2.0 - scale*(ay0+ay1)/2.0;

    let mut col = 0;
    for dact in dacts {
        col += 1;
        match dact {
            DAct::RmoveTo(x,y) => {
                let xs = scale * (x-x0);
                let ys = scale * (y-y0);
                let svgt = format!("m {:.4}in {:.4}in ",x0,y0);
                svg.push_str(&svgt);
            }
            DAct::RlineTo(x,y) => {
                let xs = scale * (x-x0);
                let ys = scale * (y-y0);
                let svgt = format!("l {:.4}in {:.4}in ",x0,y0);
                svg.push_str(&svgt);
            }
        }
        if col >= 10 {
            svg.push_str("\n");
            col = 0;
        }
    }
    if col > 0 {
        svg.push_str("\n");
    }
    svg
}

/*----------------------------------------------------------------------
Elaborate Lindenmayer System

Apply rules iteratively until specified order is reached.
Use two strings, old and new, remove character from old string,
if it is a rule, do substitution, append to new string.
Exchange old and new after each iteration.
*/

pub type Rules<'a> = HashMap<char,&'a str>;

fn rules_apply_basic(rules:&Rules, start:&str, order:i32) -> String {
    let mut new = String::from(start);
    for _ in 0..order {
        let mut old = new;
        new = "".to_string();
        while old != "" {
            let c = old.remove(0);
            match rules.get(&c) {
                Some(s) => new.push_str(s),
                None    => new.push(c),
            }
        }
    }
    new
}

/*----------------------------------------------------------------------
Higher level rule application

Apply both main rules and post rules.
The post rule substitution is used to allow use of rules from
sources that presume implicit drawing on rules other than F.
After all rule application, do minimization.
*/
fn lsys_apply_rules(lsys:&LSys,order:i32) -> String {
    // do rule substition
    let basic = rules_apply_basic(&lsys.rules,&lsys.start,order);
    // do post rule substitution
    let post = rules_apply_basic(&lsys.post_rules,&basic,1);
    rules_minimize(post)
}

/*----------------------------------------------------------------------
remove non-action characters from LSys rules
*/

fn rules_minimize(rules:String) -> String {

    let mut out = String::new();
    let actions:&str = "Ff+-[]|";
    for rule in rules.chars() {
        if actions.contains(rule) {
            out.push(rule);
        }
    }
    out
}

/*----------------------------------------------------------------------
 Work with top level lsyss
*/

#[derive(Debug, Default, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct LSys<'a> {
    title: String,
    refs:  Vec<String>,
    start: String,
    angle: f64,
    order: Vec<i32>,
    #[serde(borrow)]
    rules: Rules<'a>,
    post_rules: Rules<'a>,
}

// split json file into chunks corresponding to top level objects
// assumes that objects begin with line containing only "{"
// and end with line containing only "}"
fn json_to_chunks(json:&str) -> Vec<String> {
    let mut chunks:Vec<String> = vec!();
    let mut chunk = String::new();
    let mut inchunk:bool = false;
    for line in json.lines() {
        let l = line.trim_end();
        match (l, inchunk) {
        ("{",_) =>  {
                // begin chunk, or discard false chunk
                inchunk = true;
                chunk = "".to_string();
                chunk = chunk + line + "\n";
            }
        ("}",true) =>  {
                // end chunk
                inchunk = false;
                chunk = chunk + line + "\n";
                chunks.push(chunk.clone());
            }
        (_,true) =>  {
                // accumulate lines in chunk
                chunk = chunk + line + "\n";
            }
        (_,_) =>  {
                // ignore the rest
            }
        }
    }
    chunks
}

// load lsys from json chunks
fn lsys_from_json_chunks<'a>(chunks:&'a Vec<String>) -> Vec<LSys<'a>> {

    // iterate over chunks of lines with serde
    let mut out:Vec<LSys> = vec!();
    let mut chunk_no = 0;
    let mut errcnt = 0;
    let mut okcnt = 0;
    for chunk in chunks {
        chunk_no += 1;
        let r = serde_json::from_str::<LSys>(&chunk);
        match r {
            Err(why) => {
                errcnt += 1;
                println!();
                println!("Failed to read chunk {}",chunk_no);
                println!("--------------------------------");
                println!("{}",&chunk);
                println!("--------------------------------");
                println!("{:?}", why);
                println!();
            }
            Ok(lsys) => {
                okcnt += 1;
                //println!("{:#?}",&lsys);
                //println!("{}",&lsys.title);
                out.push(lsys);
            }
        }
    }
    println!("Successfully loaded {} of {} lsyss",
        okcnt,okcnt+errcnt);

    out
}

/*----------------------------------------------------------------------
Top level
*/

fn main() {

    // get lsys examples
    let json = include_str!("lsys_examples.json");
    let chunks = json_to_chunks(json);
    //println!("{:#?}",chunks);
    let lsys = lsys_from_json_chunks(&chunks);

    let ds = doc_new();

}
