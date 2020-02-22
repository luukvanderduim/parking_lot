#![feature(vec_remove_item)]
#[allow(dead_code)]
#[allow(unused_imports)]

// Copyright 2020 Luuk van der Duim
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

/// plot
/// 
/// Plot does progrsessive plotting during a benchmark run.
/// It can plot performance in relation to one or two variables, 
/// this results in either a 2D or a 3D chart.
/// 
/// Usage:
///  
/// plot primitive arguments axes       
/// plot [mx|rw] [Mutex|RwLock arguments] [X-axis [Y-axis]]
///
/// Note: Only arguments that are axes may be ranges!
///
/// Axes:
///
/// If mutex is opted for, axis argument(s) may be:
/// Threads [threads, T], WorkPer [per, P], WorkBetween [between, B], Seconds [secs, S] or Iterations [iters, I]
///
/// If rwlock is opted for, axis argument(s) may be:
/// Writers [W], Readers [R], WorkPer [P], WorkBetween [B], Seconds [S] or Iterations [I]
/// 
/// Some (Single letter) abreviations are accepted.
///
/// eg:
///    ```$ plot mx 1:27:3 1:10:2 1 1 3 T I```
///
/// Plots a 3D chart with threads on the x-axis and iterations on
/// the y axis.
/// 
/// ```$ plot rw 1:10:2 2 1 1 3 W ```
/// 
/// Plots a 2D chart with writer threads on the x-axis.
/// 
/// 

use core::fmt;
use core::str::FromStr;

mod args;
use args::*;

mod plotters;
use plotters::*;

use std::error::Error;
use std::io::{BufRead, BufReader};
use std::{env, process};
use std::fmt::{Debug, Display};
use std::process::{Command, Stdio};

const TRIPLET: &str = env!("TARGET");
const MANIFEST: &str = env!("CARGO_MANIFEST_DIR");

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum MxArg {
    Threads,
    WorkPer,
    WorkBetween,
    Seconds,
    Iterations,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum RwArg {
    Writers,
    Readers,
    WorkPer,
    WorkBetween,
    Seconds,
    Iterations,
}

#[derive(Clone, Copy)]
struct Mutex3D {
    x: MxArg,
    y: MxArg,
    xrange: ArgRange,
    yrange: ArgRange,
}

impl Mutex3D {
    fn get_x(&self) -> MxArg {
            self.x
    }
    fn get_y(&self) -> MxArg {
            self.y
    }
    fn get_xrange(&self) -> ArgRange {
            self.xrange
        }
    fn get_yrange(&self) -> ArgRange {
            self.yrange
        }
}

#[derive(Clone, Copy)]
struct RwLock3D {
    x: RwArg,
    y: RwArg,
    xrange: ArgRange,
    yrange: ArgRange,
}

impl RwLock3D {
    fn get_id(&self) -> &'static str {
        &"RwLock"
    }
    fn get_x(&self) -> RwArg {
        self.x
    }
    fn get_y(&self) -> RwArg {
        self.y
    }
    fn get_xrange(&self) -> ArgRange {
        self.xrange
    }
    fn get_yrange(&self) -> ArgRange {
        self.yrange
    }
}


#[derive(Clone, Copy)]
pub(crate) struct Mutex2D {
    x: MxArg,
    xrange: ArgRange,
}

impl Mutex2D {
    fn get_id(&self) -> &'static str {
        &"Mutex"
    }
    fn get_x(&self) -> MxArg {
            self.x
        }
    fn get_xrange(&self) -> ArgRange {
            self.xrange
        }
}


#[derive(Clone, Copy)]
pub(crate) struct RwLock2D {
    x: RwArg,
    xrange: ArgRange,
}

impl RwLock2D {
    fn get_id(&self) -> &'static str {
        &"RwLock"
    }
    fn get_x(&self) -> RwArg {
        self.x
    }
    fn get_xrange(&self) -> ArgRange {
        self.xrange
    }
}


trait Lockstar: Display + Debug {
    fn idx(&self) -> usize;
    fn check_corresponding_range(&self, args: &[&str]) -> Result<(), ()>;
}

impl Lockstar for MxArg {
    fn idx(&self) -> usize {
        use MxArg::*;
        match self {
            Threads => 0,
            WorkPer => 1,
            WorkBetween => 2,
            Seconds => 3,
            Iterations => 4,
        }
    }

    fn check_corresponding_range(&self, args: &[&str]) -> Result<(), ()> {
        let components = args[self.idx()].split(':').map( |e | e.trim().parse::<usize>() ).collect::<Vec<_>>();
        return { 
            match components.len() {
            2 | 3 => Ok(()),
            _ => Err(()),
            }
        }
    }
}

impl Lockstar for RwArg {
    fn idx(&self) -> usize {
        use RwArg::*;
        match self {
            Writers => 0,
            Readers => 1,
            WorkPer => 2,
            WorkBetween => 3,
            Seconds => 4,
            Iterations => 5,
        }
    }

    fn check_corresponding_range(&self, args: &[&str]) -> Result<(), ()> {
    let components = args[self.idx()].split(':').map( |e | e.trim().parse::<usize>() ).collect::<Vec<_>>();
    return match components.len() {
        2 | 3 => Ok(()),
        _ => Err(()),
    };
}
}

impl Display for MxArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use MxArg::*;
        match self {
            Threads => write!(f, "worker threads"),
            WorkPer => write!(f, "work per critical section"),
            WorkBetween => write!(f, "work between critical sections"),
            Seconds => write!(f, "duration in seconds"),
            Iterations => write!(f, "iterations"),
        }
    }
}

impl Display for RwArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RwArg::*;
        match self {
            Writers => write!(f, "writer threads"),
            Readers => write!(f, "reader threads"),
            WorkPer => write!(f, "work per critical section"),
            WorkBetween => write!(f, "work between critical sections"),
            Seconds => write!(f, "duration in seconds"),
            Iterations => write!(f, "iterations"),
        }
    }
}

impl FromStr for RwArg {
    type Err = &'static str;

    fn from_str(s: &str) -> Result< Self, Self::Err > {
        use RwArg::*;
        let s = &*s.to_ascii_lowercase();

        match s {
            "writers" | "W"                     => Ok(Writers),
            "readers" | "R"                     => Ok(Readers),
            "workper" | "per" | "P"             => Ok(WorkPer),
            "workbetween"| "between" | "B"      => Ok(WorkBetween),
            "seconds" | "secs" | "S"            => Ok(Seconds),
            "iterations" | "iters" | "I"        => Ok(Iterations),
            _                                   => Err("Invalid string for variant"),
        }
    }
}

impl FromStr for MxArg {
    type Err = &'static str;

    fn from_str(s: &str) -> Result< Self, Self::Err > {
        use MxArg::*;
        let s = &*s.to_ascii_lowercase();

        match s {
            "threads" | "t"                     => Ok(Threads),
            "workper" | "p"                     => Ok(WorkPer),
            "workbetween" | "between" | "b"     => Ok(WorkBetween),
            "seconds" | "secs" | "s"            => Ok(Seconds),
            "iterations" | "iters" | "i"        => Ok(Iterations),
            _                                   => Err("Invalid string for variant"),
        }
    }
}


/// Returns first an MxArg or RwArg str and reduced arguments list or Errs if none found
fn find_axis<'a>(args: &[&'a str] ) -> Result<(&'a str , Vec<String>) , () > {
    match args.iter().find(| arg | MxArg::from_str(arg).is_ok() || RwArg::from_str(arg).is_ok() ) {
        Some(variant) => {
            let mut argsvec: Vec<&str> = args.into(); 
            argsvec.remove_item(&variant.clone());
            Ok( ( variant , argsvec.iter().map(|i| String::from(*i) ).collect::<Vec<String>>() ))
        },
        _ => {
            Err(())
        },
    }
}



fn arg_to_range(this_arg: &[&str]) -> Result<ArgRange, ()> {
    match this_arg.len() {
        1 => Err(()),
        2 => {
            let start = this_arg[0]
                .trim()
                .parse::<usize>()
                .expect("Parsing range failed");
            let end = this_arg[1]
                .trim()
                .parse::<usize>()
                .expect("Parsing range failed");
            if start > end {
                println!("invalid range");
                process::exit(1);
            }
            Ok(args::ArgRange::new(start, end, 1))
        }
        3 => {
            let start = this_arg[0]
                .trim()
                .parse::<usize>()
                .expect("Parsing range failed");
            let end = this_arg[1]
                .trim()
                .parse::<usize>()
                .expect("Parsing range failed");
            let step = this_arg[2]
                .trim()
                .parse::<usize>()
                .expect("Parsing range failed");
            if start > end {
                println!("invalid range");
                process::exit(1);
            }
            Ok(ArgRange::new(start, end, step))
        }
        _ => Err(()),
    }
}


fn main()  {
    let mxcmd = format!("{}/target/{}/release/mutex", MANIFEST, TRIPLET);
    let rwcmd = format!("{}/target/{}/release/rwlock", MANIFEST, TRIPLET);

    let mut arguments: Vec<String> = env::args().skip(1).collect::<Vec<_>>();
    
    if arguments.is_empty() {
        eprintln!("No arguments provided.");
        process::exit(1);
    }

    // mutex or rwlock?
    if &arguments[0] == "mx" || &arguments[0] == "mutex" {
        println!("Mutex selected");
        arguments.remove(0);
        println!("Parsing args: {:?}", &arguments);
        
        // find_axis() returns, if found, an axis &str and reduced arguments Vec<String>
        if let Ok((xstr, arguments2)) = find_axis(&arguments.iter().map(|i| &**i).collect::<Vec<&str>>()) {
            println!("Found mutex x-axis argument: {}", &xstr);
            if let Ok((ystr, arguments)) = find_axis(&arguments2.iter().map(|i| &**i).collect::<Vec<&str>>()) {
            println!("Found mutex y-axis argument: {}", &ystr);

            // mutex expects 5 argumnents
            assert_eq!( arguments.iter().map(|i| &**i).collect::<Vec<&str>>().len(), 5 ); 
           
            // bind xstr and ystr to respective variants of enum MxArg
            let xaxis = MxArg::from_str(xstr).expect(&format!("Unable to match mutex arg: {} to variant", &xstr));
            println!("Check first arg {}", &xaxis);
            let yaxis = MxArg::from_str(ystr).expect(&format!("Unable to match mutex arg: {} to variant", &ystr));
            println!("Check second arg {}", &yaxis);
            
            // Check for range validity in corresponding arguments
            assert!(xaxis.check_corresponding_range(&arguments.iter().map(|i| &**i).collect::<Vec<&str>>()).is_ok()); 
            assert!(yaxis.check_corresponding_range(&arguments.iter().map(|i| &**i).collect::<Vec<&str>>()).is_ok()); 

            // Check ranges of non-axis argmuments, cannot be ranges
            assert!( !arguments.iter().enumerate().any(|(i,e)| e.contains(':') && i != xaxis.idx() && i != yaxis.idx()) );

            let xcomponents = arguments[xaxis.idx()].split(':').collect::<Vec<_>>();
            let ycomponents = arguments[yaxis.idx()].split(':').collect::<Vec<_>>();

            let m3d = Mutex3D {
                x: xaxis,
                y: yaxis,
                xrange: arg_to_range(&xcomponents).unwrap(),
                yrange: arg_to_range(&ycomponents).unwrap(),
            };

            if plot_mutex_3D(m3d, &mxcmd, &arguments.iter().map(|i| &**i).collect::<Vec<_>>()).is_err() {
                eprintln!("Could not 3D plot mutex run.")
            };
            process::exit(1);

        } else {    // We heve only found an xstr
     
            // mutex expects 5 argumnents
            assert_eq!( arguments.iter().map(|i| &**i).collect::<Vec<&str>>().len(), 5 ); 

            // Bind xaxis to corresponding variant of MxArg enum 
            let xaxis = MxArg::from_str(xstr).expect(&format!("Unable to match mutex string: {} to variant", xstr));
            
            // Check for range validity in corresponding argument
            assert!(xaxis.check_corresponding_range(&arguments.iter().map(|i| &**i).collect::<Vec<&str>>()).is_ok()); 
            
            // Check non-axes arguments, cannot be ranges
            assert!( !arguments.iter().enumerate().any(|(i,e)| e.contains(':') && i != xaxis.idx() ) );
           
            // Compose ArgRange and populate data type
            let xcomponents = arguments[xaxis.idx()].split(':').collect::<Vec<_>>();

            let m2d = Mutex2D {
                    x: xaxis,
                    xrange: arg_to_range(&xcomponents).unwrap(),
                };

            // Call plot
            if plotters::plot_mutex_2D(m2d, &mxcmd, &arguments.iter().map(|i| &**i).collect::<Vec<_>>()).is_err() {
                eprintln!("Could not plot Mutex run.");
            }
                process::exit(1);
            }
        } else {
            // No axis arguments at all, bail.
            eprintln!("No valid mutex axis arguments provided.");
            println!("Available mutex axis arguments are:\n\t'Threads'\n\t'WorkPer'\n\t'WorkBetween'\n\t'Seconds' or \n\t'Iterations'");
            process::exit(1);
        }
       
    } else if arguments[0] == "rw" || arguments[0] == "rwlock" {
        println!("RwLock selected.");
        arguments.remove(0);
        println!("Parsing args: {:?}", &arguments);
        
        // find_axis returns, if found, the &str of the axis and a reduced arguments list
        if let Ok((xstr, arguments)) = find_axis(&arguments.iter().map(|i| &**i).collect::<Vec<&str>>()) {
            println!("Found RwLock x-axis argument: {}", &xstr);
            if let Ok((ystr, arguments)) = find_axis(&arguments.iter().map(|i| &**i).collect::<Vec<&str>>()) {
            println!("Found RwLock y-axis argument: {}", &ystr);
            
            // rwlock expects 6 arguments
            assert_eq!( arguments.iter().map(|i| &**i).collect::<Vec<&str>>().len(), 6 ); 

            // We have found xstr and ystr
            let xaxis = RwArg::from_str(xstr).expect(&format!("Unable to match rwlock string: {} to variant", xstr));
            let yaxis = RwArg::from_str(ystr).expect(&format!("Unable to match rwlock string: {} to variant", ystr));
            
            // Check ranges for corresponding arguments
            assert!(xaxis.check_corresponding_range(&arguments.iter().map(|i| &**i).collect::<Vec<&str>>()).is_ok()); 
            assert!(yaxis.check_corresponding_range(&arguments.iter().map(|i| &**i).collect::<Vec<&str>>()).is_ok()); 
            
            // Check non-axes arguments, these cannot be ranges
            assert!( !arguments.iter().enumerate().any(|(i,e)| e.contains(':') && i != xaxis.idx() && i != yaxis.idx()) );
            
            // Compose ranges and populate data type
            let xcomponents = arguments[xaxis.idx()].split(':').collect::<Vec<_>>();
            let ycomponents = arguments[yaxis.idx()].split(':').collect::<Vec<_>>();

            let rw3d = RwLock3D {
                x: xaxis,
                y: yaxis,
                xrange: arg_to_range(&xcomponents).unwrap(),
                yrange: arg_to_range(&ycomponents).unwrap(),
            };

            if plot3D(rw3d, &rwcmd, &arguments.iter().map(|i| &**i).collect::<Vec<_>>()).is_err() {
                eprintln!("Could not 3D plot RwLock run");
            }
            process::exit(1);

            } else {
                // We heve found only one axis argument: xstr
                let xaxis = RwArg::from_str(xstr).expect(&format!("Unable to match rwlock string: {} to variant", xstr));

                // rwlock expects 6 arguments
                assert_eq!( arguments.iter().map(|i| &**i).collect::<Vec<&str>>().len(), 6 ); 
                
                // let's check range for corresponding argument
                assert!(xaxis.check_corresponding_range(&arguments.iter().map(|i| &**i).collect::<Vec<&str>>()).is_ok()); 

                // Check non-axes arguments, these cannot be ranges
                assert!( !arguments.iter().enumerate().any(|(i,e)| e.contains(':') && i != xaxis.idx()) );

                // Composite ArgRange and data struct
                let xcomponents = arguments[xaxis.idx()].trim().split(':').collect::<Vec<_>>();
                
                let rw2d = RwLock2D {
                    x: xaxis,
                    xrange: arg_to_range(&xcomponents).unwrap(),
                };

                // Call for plot
                if plot_rwlock_2D(rw2d, &rwcmd, &arguments.iter().map(|i| &**i).collect::<Vec<_>>()).is_err() {
                    eprintln!("Could not plot RwLock run")
                }
                process::exit(1);
            }
        } else {
            eprintln!("No rwlock arguments provided.");
            println!("Available RwLock axis arguments are:\n\t'Readers'\n\t'Writers'\n\t'WorkPer'\n\t'WorkBetween'\n\t'Seconds' or \n\t'Iterations'");
            process::exit(1);
        }
      
    }
    println!("Invalid lock argument.");
    println!("Available: mutex, mx, rwlock or rw");
}
