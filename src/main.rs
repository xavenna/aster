// Aster is a tool for simple online publishing, supporting multiple protocols.
// HTML and gemini are supported by default.

use std::fs;
use std::env;
use time;
use aster::*;


fn main() {
    let c_args = env::args();

    let mut args: Vec<String> = Vec::new();
    let mut help: bool = false;
    let mut version: bool = false;

    if env::args().len() == 1 {
        println!("Error: No arguments specified\n");
        std::process::exit(1);
    }

    let c_args: Vec<String> = c_args.collect();
    let c_args = &c_args[1..];

    for x in c_args.iter() {
        if x.len() == 0 {
            continue;
        }
        if x.len() >= 3 && &x[0..2] == "--" {
            match &x[2..] {
                "version" => {version = true;},
                "help" => {help = true;},
                _ => {eprintln!("Error: invalid argument {}",x) },
            }
        }
        else {
            args.push(x.to_string());
        }
    }


    if help {
        println!("Usage: aster [option] <infile> <outfile> <method>");
        println!("--help\t\tprint this help message.\n--version\t\tprint version info");
        return;
    } 
    if version {
        println!("aster {}\nCopyright (C) 2025 xavenna", VERSION);
        println!("This program comes with ABSOLUTELY NO WARRANTY, to the extent permitted by law.");
        println!("This is free software: you are free to change and redistribute it.");
        return;
    }

    if args.len() != 3 {
        eprintln!("Error: bad arguments.");
        std::process::exit(1);
    }
    let infile = args[0].clone();
    let outfile = args[1].clone();
    let method = args[2].clone();

    let input = match fs::read_to_string(infile) {
        Ok(s) => s,
        Err(s) => {eprintln!("{s}");return;},
    };
    let a = parse_article(&input).unwrap();

    let result = match method.trim() {
        "html" => gen_html(a),
        "gmi" => gen_gemtext(a),
        _ => {eprintln!("Error: bad method");std::process::exit(1);},

    }.unwrap();

    //write to a file
    match fs::write(&outfile, &result) {
        Ok(_) => {},
        Err(s) => {eprintln!("Could not write to output file: {s}");return;},
    }
}

/// turns an Article struct to gemtext
fn gen_gemtext(mut art: Article) -> Result<String, XVError> {
    //load the template
    let template = match fs::read_to_string("template.gmi") {
        Ok(s) => s,
        Err(s) => {eprintln!("{s}");return Err(XVError::from("Couldn't load templ"));},
    };

    //process body here? (Split into paragraphs, delete single newlines 
    let out = art.body.clone();
    
    let lines: Vec<String> = out.lines().map(|x| x.to_string()).collect();
    
    let mut outlines: Vec<String> = Vec::new();
    let mut line = String::new();

    for x in lines {
        if x.is_empty() {
            if !line.is_empty() {
                outlines.push(line);
                line = String::new();
            }
        } else {
            line += &x;
            line.push(' ');
        }

    }

    //now, add in references. Search each line for references, then remove them. After
    //each paragraph, insert the appropriate links
    let mut ind: Vec<usize> = Vec::new();
    for x in &outlines {
        let mut pos = 0;
        loop {
            let subs = &x[pos..];
            if let Some(n) = subs.find('[') {
                if let None = subs.find(']') {
                    return Err(XVError::from("No closing ']' found"));
                }
                let p = subs.find("]").unwrap();
                let name = &subs[n+1..p];
                pos = p+1;
                let id = match name.parse::<usize>() {
                    Ok(s) => s,
                    Err(s) => {return Err(XVError::from(s.to_string()));},
                };
                if id > ind.len()+1 {
                    return Err(XVError::from(format!("Annotation '{}' skipped", id-1)));
                }
                ind.push(id);
            } else {
                break;
            }
        }
    }

    //render outlines into body
    art.body = String::new();
    for x in outlines {
        art.body += &x;
        art.body += "\n\n";

    }

    //go through template, look for $$tags$_
    
    let mut out = String::new();
    let mut pos: usize = 0;
    loop {  // FIX OFF BY ONE, OFF BY TWO ERRORS HERE !!!!!!!!!!!!!!!!!!!!!!!!!!
        let subs = &template[pos..];
        //get template from pos until next instance of "##"
        if let Some(n) = subs.find("$$") {
            if let None = subs.find("$_") {
                return Err(XVError::from("No closing tag found"));
            }
            let p = subs.find("$_").unwrap();

            let name = &subs[n+2..p];
            //eprintln!("{},{},{},{}", name, pos, n, p);

            let text = &subs[..n];
            pos += text.len() + 4 + name.len();

            out += text;
            //now, resolve key's value 
            let val = match gmi_key(name, &art) {
                Some(s) => s,
                None => {return Err(XVError::from(format!("Invalid subst '{}'",name)))},
            };
            out += &val;

        } else {
            //no more tags. break at the end of this operation
            out += subs;
            break;
        }

    }


    Ok(out)
}

/// turns an Article struct to html
fn gen_html(mut art: Article) -> Result<String, XVError> {
    let mut out = String::new();
    //load the template, and substitute the tags
    let template = match fs::read_to_string("template.html") {
        Ok(s) => s,
        Err(s) => {eprintln!("{s}");return Err(XVError::from("Couldn't load templ"));},
    };

    //process the body here (wrap footnotes in <a> tags, replace repeated \n with <br />
    let mut pos: usize = 0;
    loop {  // FIX OFF BY ONE, OFF BY TWO ERRORS HERE !!!!!!!!!!!!!!!!!!!!!!!!!!
        let subs = &art.body[pos..];
        //get template from pos until next instance of "##"
        if let Some(n) = subs.find("[") {
            if let None = subs.find("]") {
                return Err(XVError::from("No closing ] found"));
            }
            let p = subs.find("]").unwrap();

            let name = &subs[n+1..p];

            let text = &subs[..n];
            pos += text.len() + 2 + name.len();

            out += text;
            //now, resolve key's value 
            let tag = format!("<a id=\"retnote{}\" href=\"#note{}\">[{}]</a>",name,name,name);
            out += &tag;

        } else {
            //no more tags. break at the end of this operation
            out += subs;
            break;
        }
    }
    //replace any duplicated \n with <br />
    //eventually, construct <p> around paragraphs
    out = out.replace("\n\n", "<br /><br />");
    art.body = out;

    let mut out = String::new();
    let mut pos: usize = 0;

    loop {  // FIX OFF BY ONE, OFF BY TWO ERRORS HERE !!!!!!!!!!!!!!!!!!!!!!!!!!
        let subs = &template[pos..];
        //get template from pos until next instance of "##"
        if let Some(n) = subs.find("##") {
            if let None = subs.find("#_") {
                return Err(XVError::from("No closing tag found"));
            }
            let p = subs.find("#_").unwrap();

            let name = &subs[n+2..p];
            eprintln!("{},{},{},{}", name, pos, n, p);


            let text = &subs[..n];
            pos += text.len() + 4 + name.len();

            out += text;
            //now, resolve key's value 
            let val = match html_key(name, &art) {
                Some(s) => s,
                None => {return Err(XVError::from(format!("Invalid subst '{}'",name)))},
            };
            out += &val;

        } else {
            //no more tags. break at the end of this operation
            out += subs;
            break;
        }

    }

    //dbg!(&out);

    Ok(out)
}

fn html_key(name: &str, art: &Article) -> Option<String> {
    match name {
        "title" => Some(art.title.clone()),
        "subtitle" => Some(art.subtitle.clone()),
        "body" => Some(art.body.clone()),
        "notes" => Some(html_notes(art)),
        "date" => {
            let now = time::UtcDateTime::now().date().to_string();
            Some(now)

        },
        "version" => Some(VERSION.to_string()),
        _ => None,

    }
}

fn gmi_key(name: &str, art: &Article) -> Option<String> {
    match name {
        "title" => Some(art.title.clone()),
        "subtitle" => Some(art.subtitle.clone()),
        "body" => Some(art.body.clone()),
        "notes" => Some(gmi_notes(art)),
        "date" => {
            let now = time::UtcDateTime::now().date().to_string();
            Some(now)

        },
        "version" => Some(VERSION.to_string()),
        _ => None,

    }
}
fn gmi_notes(art: &Article) -> String {
    let mut out = String::new();
    for (i, x) in art.notes.iter().enumerate() {
        let j=i+1;
        //eventually, make this detect links
        let p = format!("* [{}] : {}\n", j, x);
        out += &p;
    }
    out
}


fn html_notes(art: &Article) -> String {
    let mut out = String::new();
    for (i, x) in art.notes.iter().enumerate() {
        let j=i+1;
        let p = format!("<p id=\"note{}\">[{}]  {}  <a href=\"#retnote{}\">(return)</a></p>\n", j, j, x, j);
        out += &p;
    }
    out
}

/// turns an article, stored as text, into an Article struct
fn parse_article(text: &String) -> Result<Article,XVError> {
    //extract dirs: title, subtitle, tags, footnotes
    //
    let mut art = Article::new();
    
    //split string into lines
    let lines: Vec<String> = text.lines().map(|x| x.to_string()).collect();
    
    let mut body = String::new();
    for l in lines {
        //if line begins with #, parse it
        if l.len() != 0 {
            if l[0..1] == *"#" {
                if let Some(n) = l.find(':') {
                    let key = &l[1..n];
                    let val = &l[(n+1)..].to_string();
                    if key == "title" {
                        art.title = val.to_string();

                    } else if key == "subtitle" {
                        art.subtitle = val.to_string();

                    } else if key == "tags" {
                        //parse val by commas
                        let tags: Vec<String> = parse(val, ',');
                        for x in tags.iter() {
                            if let Some(s) = Tag::from(x) {
                                art.tags.push(s);
                            } else {
                                return Err(XVError::from("Invalid tag"));
                            }
                        }

                    } else if let Ok(i) = key.parse::<usize>() {
                        // footnote #i
                        //check if specified footnote has been declared yet?
                        let pos = art.notes.len();
                        if i != pos+1  { //Footnote num is one off len
                            return Err(XVError::from("Invalid footnote number"));
                        }
                        art.notes.push(val.to_string());

                    } else {
                        //invalid
                        return Err(XVError::from("Invalid directive"));

                    }

                } else {
                    return Err(XVError::from(format!("Error: Null tag")));
                }
                //parse tag
                //dont add newline
            } else {
                body += &l;
                body += "\n";
            }

        } else {
            body += "\n";
        }
    }
    
    art.body = body;

    Ok(art)
}


