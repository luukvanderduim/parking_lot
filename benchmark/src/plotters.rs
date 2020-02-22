#![allow(non_snake_case)]
use super::*;

use progress_bar::progress_bar::ProgressBar;
use progress_bar::color::Color as BarColor;

use progress_bar::color::Style;
use itertools::Itertools;

use gnuplot::*;
use gnuplot::{Caption, Color, Figure};


pub(crate) fn plot_mutex_2D(data: Mutex2D, cmd: &str, args: &[&str]) -> Result<(), Box<dyn Error>>  { 
    let mut fg = Figure::new();

    let xarg = data.get_x();
    let primitive_name = data.get_id();
    
    let len = data.get_xrange().clone().count();
    let ranger = data.get_xrange().clone();
    let title = format!("{} locking performance as a function of {}", primitive_name, xarg);
    
    let mut d: Vec<f64> = vec![0.0; len * 3];
    let mut vidx: usize = 0;
           
    fg.set_terminal("wxt", "plot_mutex_2D");
    fg.set_title(&title);
    
    let stdout = Command::new(cmd)
    .args(args)
    .stdout(Stdio::piped())
    .spawn()?
    .stdout 
    .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other,"Could not capture standard output."))?;

            BufReader::new(stdout).lines().map( |l| l.expect("No line found"))
            .filter( |line| line.find("kHz").is_some())
            .for_each( | e | {            
             
                    d[vidx] = e.trim().split_whitespace().filter_map( |word| word.parse::<f64>().ok()).step_by(3).next().expect("No f64 found");
                    vidx += 1;

                if vidx % 3 == 0 || (d.len() - vidx) < 3  {
                fg.clear_axes();
                fg.axes2d()
                .set_y_range(Auto, Auto)
                .set_x_label(&format!("{}", &xarg), &[])
                .set_y_label("kHz", &[Rotate(90.0)])
                .set_legend(Graph(0.8), Graph(0.95), &[], &[])
                .lines(
                        ranger.clone().into_iter(),
                        d[0..].iter().step_by(3),
                        &[Caption("Parking lot"), Color("red") , LineWidth(2.0)],
                    )
                .lines(
                        ranger.clone().into_iter(),
                        d[1..].iter().step_by(3),
                        &[Caption("std library"), Color("blue") , LineWidth(2.0)],
                    )
                .lines(
                        ranger.clone().into_iter(),
                        d[2..].iter().step_by(3),
                        &[Caption("pthread"), Color("black") , LineWidth(2.0)],
                    );
                fg.show().unwrap(); 
                } 

            } );
    Ok(()) 
}

pub(crate) fn plot_rwlock_2D(data: RwLock2D, cmd: &str, args: &[&str]) -> Result<(), Box<dyn Error>>  {
    
    let mut fg = Figure::new();

    let xarg = &data.get_x();
    let primitive_name = data.get_id();
        
    let ranger = &data.get_xrange().clone();
    let title = format!("{} locking performance as a function of {}", primitive_name, xarg);
    
    let len = &data.get_xrange().clone().count();
    let mut d: Vec<f64> = Vec::with_capacity(len * 8);
    let mut vidx: usize = 0;
           
    fg.set_terminal("", "plot_rwlock_2D");
    fg.set_title(&title);
    fg.set_multiplot_layout(2, 2) // Two rows, two columns
        .set_title("RwLock: parking_lot performance compared to std, seqlock and pthread")
		.set_scale(0.8, 0.8)
		.set_offset(0.0, 0.0);

    let stdout = Command::new(cmd)
    .args(args)
    .stdout(Stdio::piped())
    .spawn()?
    .stdout 
    .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other,"Could not capture standard output."))?;
    
            BufReader::new(stdout)
                    .lines()  // takes ownership lines(self) -> Lines<Self>
                    .map( | l | l.expect("No line found"))
                    .filter( | line | line.find("kHz").is_some() )
                    .inspect(|a| println!("{}", &a) )
                    .map( | s | s.split_whitespace().map(str::to_owned).collect::<Vec<_>>() ) // s is by value, borrowed by split_whitespace
                    .flat_map( | word | word.into_iter().flat_map(|e| e.parse::<f64>()) )
                    .for_each( |f| {
                
                d.push(f); // d[vidx] = f; is a bug, but why?
                
                vidx += 1;
                
                if vidx % 8 == 0 {
                fg.clear_axes();

                fg.axes2d()
                .set_title("parking lot vs SeqLock", &[] )
                .set_y_range(Auto, Auto)
                .set_x_label(&format!("{}", &xarg), &[])
                .set_y_label("kHz", &[Rotate(90.0)])
                .set_legend(Graph(0.9), Graph(0.95), &[], &[])
                .lines_points(
                        ranger.clone().into_iter(),
                        d[0..].iter().step_by(8),
                        &[Caption("parking lot writes"), Color("red") , LineWidth(2.0), PointSymbol('s')],
                    )
                .lines_points(
                        ranger.clone().into_iter(),
                        d[1..].iter().step_by(8),
                        &[Caption("parking lot reads"), Color("red") , LineWidth(2.0), PointSymbol('S')],
                    )
                .lines_points(
                        ranger.clone().into_iter(),
                        d[2..].iter().step_by(8),
                        &[Caption("SeqLock writes"), Color("purple") , LineWidth(2.0), PointSymbol('o')],
                    )
                .lines_points(
                        ranger.clone().into_iter(),
                        d[3..].iter().step_by(8),
                        &[Caption("SeqLock reads"), Color("purple") , LineWidth(2.0), PointSymbol('O')],
                    );
                
                 fg.axes2d()
                    .set_title("parking lot vs std RwLock", &[] )
                    .set_y_range(Auto, Auto)
                    .set_x_label(&format!("{}", &xarg), &[])
                    .set_y_label("kHz", &[Rotate(90.0)])
                    .set_legend(Graph(0.9), Graph(0.95), &[], &[])
                .lines_points(
                        ranger.clone().into_iter(),
                        d[0..].iter().step_by(8),
                        &[Caption("parking lot writes"), Color("red") , LineWidth(2.0), PointSymbol('s')],
                    )
                .lines_points(
                        ranger.clone().into_iter(),
                        d[1..].iter().step_by(8),
                        &[Caption("parking lot reads"), Color("red") , LineWidth(2.0), PointSymbol('S')],
                    )
                .lines_points(
                        ranger.clone().into_iter(),
                        d[4..].iter().step_by(8),
                        &[Caption("std writes"), Color("blue") , LineWidth(2.0), PointSymbol('o')],
                    )
                .lines_points(
                        ranger.clone().into_iter(),
                        d[5..].iter().step_by(8),
                        &[Caption("std reads"), Color("blue") , LineWidth(2.0), PointSymbol('O')],
                    );

                fg.axes2d()
                        .set_title("parking lot vs pthread RwLock", &[] )
                        .set_y_range(Auto, Auto)
                        .set_x_label(&format!("{}", &xarg), &[])
                        .set_y_label("kHz", &[Rotate(90.0)])
                        .set_legend(Graph(0.9), Graph(0.95), &[], &[])
                    .lines_points(
                        ranger.clone().into_iter(),
                        d[0..].iter().step_by(8),
                        &[Caption("parking lot writes"), Color("red") , LineWidth(2.0), PointSymbol('s')],
                    )
                    .lines_points(
                        ranger.clone().into_iter(),
                        d[1..].iter().step_by(8),
                        &[Caption("parking lot reads"), Color("red") , LineWidth(2.0), PointSymbol('S')],
                    )
                    .lines_points(
                        ranger.clone().into_iter(),
                        d[6..].iter().step_by(8),
                        &[Caption("pthread writes"), Color("black") , LineWidth(2.0), PointSymbol('o')],
                    )
                    .lines_points(
                        ranger.clone().into_iter(),
                        d[7..].iter().step_by(8),
                        &[Caption("pthread reads"), Color("black") , LineWidth(2.0), PointSymbol('O')],
                    ); 

                fg.show().unwrap();
                }  
            } );
    Ok(()) 
}

pub(crate) fn plot_mutex_3D(data: Mutex3D, cmd: &String, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    
    let xarg = data.get_x();
    let yarg = data.get_y();

    let primitive_name = "mutex";
        
    let xrange = data.get_xrange().len();
    let yrange = data.get_yrange().len();
    let size = xrange * yrange;

    let mut pl_storage:     Vec<f64> = vec![0.0f64; size]; // parking lot storage
    let mut std_storage:    Vec<f64> = vec![0.0f64; size];  // std::sync::mutex storage
    let mut pt_storage:     Vec<f64> = vec![0.0f64; size];  // pthread mutex storage

    let mut fg = Figure::new();

    // Silly, TODO: Needs a fix
    // Need to set file name to make Gnuplot stop from being overly noisy
    // The empty str slice yields complaints:
    fg.set_terminal("wxt", "mutex_3d"); 
    
    let title = format!("{}' performance as functions of {} and {}", primitive_name, xarg, yarg);
    fg.set_title(&title);

    fg.set_multiplot_layout(2, 2) // Two rows, two columns
        .set_offset(0.0, 0.0);
        
        
    let mut pb = ProgressBar::new(size);


    let cube: (f64, f64, f64, f64) = {  let (xcur, xlim, _xstep) = data.get_xrange().get_by_values();
                                        let (ycur, ylim, _ystep) = data.get_yrange().get_by_values();
                                        (xcur as f64, ycur as f64, xlim as f64, ylim as f64) 
                                    };

                                        
    
    let stdout = Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .spawn()?
            .stdout 
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other,"Could not capture standard output."))?;

    BufReader::new(stdout)
            .lines()
            .map( |l| l.expect("No line found"))
            .filter( |line| line.find("kHz").is_some())
            .map( |s| s.split_whitespace().map(str::to_owned).collect::<Vec<_>>() ) // s is by value, borrowed by split_whitespace
            .flat_map( | word | word.into_iter().flat_map(|e| e.parse::<f64>()) )
            .tuples() //  Yay itertools! ;)
            .enumerate()
            // .inspect(| ( index, ( a, b, c  )) | println!("index: {} - pl: {}\t std: {}\t pt: {}", index, a, b, c ))
            .for_each( | (index, ( pl_rsl, std_rsl, pt_rsl)) | { 
                
                let mut update_chart =  |pl_storage: Vec<f64>, std_storage: Vec<f64>, pt_storage: Vec<f64>| {
                    fg.clear_axes();
                
                fg.axes3d()
                    .set_title(&format!("parking lot : conditions {} and {}", &xarg, &yarg), &[])
                    .surface( pl_storage.iter(), xrange, yrange, Some(cube), &[])
                    .set_x_label(&format!("{}", &xarg), &[])
                    .set_y_label(&format!("{}", &yarg), &[Rotate(-45.0)])
                    .set_z_label("kHz", &[Rotate(90.0)])
                    .set_z_range(Auto, Auto)
                    .set_z_ticks(Some((AutoOption::Auto, 1)), &[], &[])
                    .set_view(70.0, 75.0);

                    
                fg.axes3d()
                    .set_title(&format!("std : conditions {} and {}", &xarg, &yarg) , &[])
                    .surface(std_storage.iter(), xrange, yrange, Some(cube), &[])
                    .set_x_label(&format!("{}", &xarg), &[Rotate(-45.0)] )
                    .set_y_label(&format!("{}", &yarg), &[] )
                    .set_z_label("kHz", &[Rotate(90.0)])
                    .set_z_range(Auto, Auto)
                    .set_z_ticks(Some((AutoOption::Auto, 1)), &[], &[])
                    .set_view(70.0, 75.0);               

                fg.axes3d()
                    .set_title(&format!("pthread : conditions {} and {}", &xarg, &yarg), &[])
                    .surface( pt_storage.iter(), xrange, yrange, Some(cube), &[])
                    .set_x_label(&format!("{}", &xarg), &[])
                    .set_y_label(&format!("{}", &yarg), &[Rotate(-45.0)])
                    .set_z_label("kHz", &[Rotate(90.0)])
                    .set_z_range(Auto, Auto)
                    .set_z_ticks(Some((AutoOption::Auto, 1)), &[], &[])
                    .set_view(70.0, 75.0);
                
                fg.show().expect("Unable to show mutex 3D plot");
                };

                pb.set_action("Acquiring..", BarColor::Blue, Style::Bold);
                pb.inc();

                println!("index: {} :: \r", index);
  
                pl_storage[index] = pl_rsl;
                std_storage[index] = std_rsl;
                pt_storage[index] = pt_rsl;


               // if index % yrange == 0 {
                    update_chart(pl_storage.clone(), std_storage.clone(), pt_storage.clone());
                //}
            }); 
    Ok(())   
}

pub(crate) fn plot3D(data: RwLock3D, cmd: &String, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {

    Ok(())
}
