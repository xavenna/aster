use std::fmt;
use std::fs;

pub const VERSION: &str = "v.pre";

// Splits a string up to a vector by delim
pub fn parse(st: &str, delim: char) -> Vec<String> {
    let mut th: Vec<String> = Vec::new();
    let mut in_str: bool = false;
    let mut cw = String::new();
    
    for ch in st.chars() {
        if ch == delim {
            if in_str {
                cw.push(ch);
            } else {
                if !cw.is_empty() {
                    th.push(cw);
                    cw = String::new();
                }
            }
        }
        else if ch == '"' {
            cw.push('"');
            in_str = !in_str;

        } else {
            cw.push(ch);
        }

    }
    //the final piece won't be added.
    if !cw.is_empty() {
        th.push(cw);
    }
    th
}

// for clean error handling:
#[derive(Debug)]
pub struct XVError {
    msg: String,
}
impl std::fmt::Display for XVError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg) // user-facing output
    }
}
impl XVError {
    pub fn from_slice(st: &str) -> XVError {
        XVError {
            msg: st.to_string(),
        }
    }
}
impl From<String> for XVError {
    fn from(st: String) -> Self {
        XVError {
            msg: st.clone(),
        }
    }
}
impl From<&str> for XVError {
    fn from(st: &str) -> Self {
        XVError {
            msg: st.to_string(),
        }
    }
}

// misc utils

pub fn read_file(infile: &String) -> Option<String> {
    let r = fs::read_to_string(infile);
    match r {
        Ok(s) => Some(s),
        Err(_) => None,
    }
}



// begin the proper code

#[derive(Debug,PartialEq)]
pub enum Tag {
    Story,
    Project,
    Music,
    Review,
    Misc,
}

impl Tag {
    pub fn from(st: &String) -> Option<Tag> {
        match st.trim() {
            "story" => Some(Tag::Story),
            "project" => Some(Tag::Project),
            "music" => Some(Tag::Music),
            "review" => Some(Tag::Review),
            "misc" => Some(Tag::Misc),
            _ => None,
        }
    }
}

/// Contains the basic article. No formatting has been applied.
#[derive(Debug)]
pub struct Article {
    pub title: String,
    pub subtitle: String,
    pub tags: Vec<Tag>,
    pub notes: Vec<String>,
    pub body: String,
}

impl Article {
    pub fn new() -> Article {
        Article {
            title: String::new(),
            subtitle: String::new(),
            tags: Vec::new(),
            notes: Vec::new(),
            body: String::new(),
        }
    }

}


