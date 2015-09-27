#![allow(unused_mut)]

use std::collections::HashMap;
use std::borrow::ToOwned;
use std::cmp;

use doc::*;
use stepper::*;
use compose;


#[derive(Clone, Debug)]
pub struct AddStepper {
    pub head:Option<AddElement>,
    pub rest:Vec<AddElement>,
    pub stack:Vec<Vec<AddElement>>,
}

impl AddStepper {
    pub fn new(span:&AddSpan) -> AddStepper {
        let mut ret = AddStepper {
            head: None,
            rest: span.to_vec(),
            stack: vec![],
        };
        ret.next();
        ret
    }

    pub fn next(&mut self) -> Option<AddElement> {
        let res = self.head.clone();
        self.head = if self.rest.len() > 0 { Some(self.rest.remove(0)) } else { None };
        res
    }

    pub fn get_head(&self) -> AddElement {
        self.head.clone().unwrap()
    }

    pub fn is_done(&self) -> bool {
        self.head.is_none() && self.stack.len() == 0
    }

    pub fn enter(&mut self) {
        let head = self.head.clone();
        self.stack.push(self.rest.clone());
        let span = match head {
            Some(AddGroup(_, ref span)) |
            Some(AddWithGroup(ref span)) => {
                self.head = None;
                self.rest = span.to_vec();
                self.next();
            },
            _ => {
                panic!("Entered wrong thing")
            }
        };
    }

    pub fn exit(&mut self) {
        let last = self.stack.pop().unwrap();
        self.rest = last;
        self.next();
    }
}

#[derive(Clone, Debug)]
pub struct AddWriter {
    pub past:Vec<AddElement>,
    stack: Vec<Vec<AddElement>>,
}

impl AddWriter {
    pub fn new() -> AddWriter {
        AddWriter {
            past: vec![],
            stack: vec![],
        }
    }

    pub fn begin(&mut self) {
        let past = self.past.clone();
        self.past = vec![];
        self.stack.push(past);
    }

    pub fn exit(&mut self) {
        let past = self.past.clone();
        self.past = self.stack.pop().unwrap();
        self.past.push(AddWithGroup(past));
    }

    pub fn close(&mut self, attrs: Attrs) {
        let past = self.past.clone();
        self.past = self.stack.pop().unwrap();
        self.past.push(AddGroup(attrs, past));
    }

    pub fn skip(&mut self, n: usize) {
        compose::add_place_any(&mut self.past, &AddSkip(n));
    }

    pub fn chars(&mut self, chars: &str) {
        compose::add_place_any(&mut self.past, &AddChars(chars.into()));
    }

    pub fn result(self) -> AddSpan {
        if self.stack.len() > 0 {
            println!("{:?}", self);
            assert!(false, "cannot get result when stack is still full");
        }
        self.past
    }
}







#[derive(PartialEq, Clone)]
enum TrackType {
    NoType,
    TextBlock,
}

#[derive(Clone, Debug)]
struct Track {
    tag_a: Option<String>,
    tag_real: Option<String>,
    tag_b: Option<String>,
    is_original_a: bool,
    is_original_b: bool,
}

fn get_type(attrs:&Attrs) -> TrackType {
    TrackType::TextBlock    
}

struct Transform {
    tracks: Vec<Track>,
    a_add: AddWriter,
}

impl Transform {
    // fn use() {

    fn enter(&mut self, name:String) {
    //   iterA.apply(insrA);
    //   iterA.apply(insrB);
    //   delrA.enter();
    //   delrB.enter();
        self.tracks.push(Track {
            tag_a: Some(name.clone()),
            tag_real: Some(name.clone()),
            tag_b: Some(name.clone()),
            is_original_a: true,
            is_original_b: true,
        });

        self.a_add.begin();
    }

    // Close the topmost track.
    fn abort(&mut self) -> (Option<String>, Option<String>, Option<String>) {
        let track = self.tracks.pop().unwrap();
        println!("ABORTIN {:?}", track);
        if let Some(ref real) = track.tag_real {
            // if track.tag_a.is_some() {
            self.a_add.close(container! { ("tag".into(), real.clone() )}); // fake
            // } else {
            //     self.a_add.close(container! { ("tag".into(), track.tag_a.into() )}); // fake
            // }
            // if (a) {
            //   insrA.alter(r, {}).close();
            // } else {
            //   insrA.close();
            // }
            // if (b) {
            //   insrB.alter(r, {}).close();
            // } else {
            //   insrB.close();
            // }
        }
        (track.tag_a, track.tag_real, track.tag_b)
    }

    fn skip_a(&mut self, n: usize) {
        self.a_add.skip(n);
    }

    // fn skip_b(&mut self, n: usize) {
    //     self.b_add.skip(n);
    // }

    fn current(&self) -> Option<Track> {
        let value = self.tracks.last();
        if let Some(track) = value {
            Some((*track).clone())
        } else {
            None
        }
    }

    // Interrupt all tracks up the ancestry until we get to
    // a particular type, OR a type than could be an ancestor
    // of the given type
    fn interrupt(&mut self, itype:TrackType) {
        let mut regen = vec![];
        loop {
            let mut value = None;
            {
                if let Some(..) = self.current() {
                    value = self.current().clone();
                }
            }

            if let Some(track) = value {
                if track.tag_real.is_some() {
                    // schema.findType(tran.current()[1]) != type && schema.getAncestors(type).indexOf(schema.findType(tran.current()[1])) == -1
                    let aborted = self.abort();
                    regen.push(aborted);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        for group in regen {
            self.tracks.push(Track {
                tag_a: group.0,
                tag_real: None,
                tag_b: group.2,
                is_original_a: false,
                is_original_b: false,
            })
        }
    }

    fn regenerate(&mut self) {
        // okay do regen
        // Filter for types that are ancestors of the current type.
        // TODO
        println!("DOING THAT REGEX");
        for track in self.tracks.iter_mut() {
            if track.tag_real.is_none() {
                if track.tag_a.is_some() || track.tag_b.is_some() {
                    track.tag_real = track.tag_a.clone();

                    self.a_add.begin();

                    // if (origA) {
                    //   insrA.enter();
                    // } else {
                    //   insrA.open(a || b, {});
                    // }
                    
                    // if (origB) {
                    //   insrB.enter();
                    // } else {
                    //   insrB.open(a || b, {});
                    // }
                }
            }
        }
    }

    fn result(mut self) -> AddSpan {
        let mut span = self.a_add;
        for track in self.tracks.iter_mut().rev() {
            println!("TRACK RESULT: {:?}", track);
            if !track.is_original_a {
                span.close(container! { ("tag".into(), track.tag_a.clone().unwrap() )});
            } else {
                span.exit();
            }
        }
        span.result()
    }
}

fn transform_insertions(avec:&AddSpan, bvec:&AddSpan) -> AddSpan {
    // let mut res = Vec::with_capacity(avec.len() + bvec.len());


    let mut a = AddStepper::new(avec);
    let mut b = AddStepper::new(bvec);

    let mut t = Transform {
        tracks: vec![],
        a_add: AddWriter::new(),
    };

    let mut a_type = TrackType::NoType;
    let mut b_type = TrackType::NoType;

    while !(a.is_done() && b.is_done()) {
        println!("FACED WITH {:?} {:?}", a.head, b.head);

        if a.is_done() || b.is_done() {
            t.regenerate();

            if a.is_done() {
                match b.head.clone() {
                    Some(AddSkip(b_count)) => {
                        t.skip_a(b_count);
                        b.next();
                    },
                    None => {
                        b.exit();
                    },
                    _ => {
                        panic!("What");
                    }
                }
            }

        } else {
            match (a.head.clone(), b.head.clone()) {
                (Some(AddGroup(ref a_attrs, _)), Some(AddGroup(ref b_attrs, _))) => {
                    a_type = get_type(a_attrs);
                    b_type = get_type(b_attrs);

                    if a_type == b_type {
                        println!("My");
                    }

                    a.enter();
                    b.enter();
                    t.enter(a_attrs.get("tag").unwrap().clone())
                },
                (Some(AddSkip(a_count)), Some(AddSkip(b_count))) => {
                    if a_count > b_count {
                        a.head = Some(AddSkip(a_count - b_count));
                        b.next();
                    } else if a_count < b_count {
                        a.next();
                        b.head = Some(AddSkip(b_count - a_count));
                    } else {
                        a.next();
                        b.next();
                    }
                    t.skip_a(::std::cmp::min(a_count, b_count));
                },
                (None, None) => {
                    a.exit();
                    b.exit();
                },
                (None, Some(AddSkip(b_count))) => {
                    t.interrupt(a_type.clone());
                    // t.closeA()
                    a.exit()
                },
                (Some(AddSkip(a_count)), None) => {
                    t.interrupt(b_type.clone());
                    // t.closeA()
                    b.exit()
                },
                (Some(AddChars(ref a_chars)), _) => {
                    // do some chars
                },
                _ => {
                    panic!("No idea: {:?}, {:?}", a.head, b.head);
                },
            }
        }
    }

    // Ugh really
    // vec![
    //     AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(4)]),
    //     AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(2)])
    // ]

    t.result()
}

#[test]
fn test_transform_goose() {
    assert_eq!(transform_insertions(&vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(4)])
    ], &vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(6)])
    ]), vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(4)]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(2)])
    ]);
}

// #[test]
// fn test_transform_cory() {
//     assert_eq!(transform_insertions(&vec![
//         AddSkip(1), AddChars("1".into())
//     ], &vec![
//         AddSkip(1), AddChars("2".into())
//     ]), vec![
//         AddSkip(1), AddChars("12".into()),
//     ]);
// }
