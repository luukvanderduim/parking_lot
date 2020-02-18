#![allow(non_snake_case)]
use super::*;

pub(crate) fn plot_mutex_2D(data: Mutex2D, cmd: &str, args: &[&str]) -> Result<(), Box<dyn Error>>  {
    
    let mut fg = Figure::new();

    let xarg = data.get_x();
    let primitive_name = data.get_id();
    
    let len = data.get_xrange().clone().count();
    let ranger = data.get_xrange().clone();
    let title = format!("{} locking performance as a function of {}", primitive_name, xarg);

    
    let mut d: Vec<f64> = vec![0.0; len * 3];
    let mut vidx: usize = 0;
           
    fg.set_terminal("", "plot_mutex_2D");
    fg.set_title(&title);
    

    let stdout = Command::new(cmd)
    .args(args)
    .stdout(Stdio::piped())
    .spawn()?
    .stdout 
    .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other,"Could not capture standard output."))?;

            BufReader::new(stdout).lines().map( |l| l.expect("No line found"))
            .filter( |line| line.find("kHz").is_some())
            .for_each( |e| { 
             
                    d[vidx] = e.trim().split_whitespace().filter_map( |word| word.parse::<f64>().ok()).step_by(3).next().expect("No f64 found");
                    dbg!(vidx += 1);

                if vidx % 3 == 0 {
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
                .set_title("parking_lot vs SeqLock", &[] )
                .set_y_range(Auto, Auto)
                .set_x_label(&format!("{}", &xarg), &[])
                .set_y_label("kHz", &[Rotate(90.0)])
                .set_legend(Graph(0.9), Graph(0.95), &[], &[])
                .lines_points(
                        ranger.clone().into_iter(),
                        d[0..].iter().step_by(8),
                        &[Caption("parking_lot writes"), Color("red") , LineWidth(2.0), PointSymbol('s')],
                    )
                .lines_points(
                        ranger.clone().into_iter(),
                        d[1..].iter().step_by(8),
                        &[Caption("parking_lot reads"), Color("red") , LineWidth(2.0), PointSymbol('S')],
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
                    .set_title("parking_lot vs std_RwLock", &[] )
                    .set_y_range(Auto, Auto)
                    .set_x_label(&format!("{}", &xarg), &[])
                    .set_y_label("kHz", &[Rotate(90.0)])
                    .set_legend(Graph(0.9), Graph(0.95), &[], &[])
                .lines_points(
                        ranger.clone().into_iter(),
                        d[0..].iter().step_by(8),
                        &[Caption("parking_lot writes"), Color("red") , LineWidth(2.0), PointSymbol('s')],
                    )
                .lines_points(
                        ranger.clone().into_iter(),
                        d[1..].iter().step_by(8),
                        &[Caption("parking_lot reads"), Color("red") , LineWidth(2.0), PointSymbol('S')],
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
                        .set_title("parking_lot vs pthread_RwLock", &[] )
                        .set_y_range(Auto, Auto)
                        .set_x_label(&format!("{}", &xarg), &[])
                        .set_y_label("kHz", &[Rotate(90.0)])
                        .set_legend(Graph(0.9), Graph(0.95), &[], &[])
                    .lines_points(
                        ranger.clone().into_iter(),
                        d[0..].iter().step_by(8),
                        &[Caption("parking_lot writes"), Color("red") , LineWidth(2.0), PointSymbol('s')],
                    )
                    .lines_points(
                        ranger.clone().into_iter(),
                        d[1..].iter().step_by(8),
                        &[Caption("parking_lot reads"), Color("red") , LineWidth(2.0), PointSymbol('S')],
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

pub(crate) fn plot3D<T: LockData3D>(data: T, cmd: &String, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let stdout = Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .spawn()?
            .stdout
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other,"Could not capture standard output." ));
    Ok(())
}