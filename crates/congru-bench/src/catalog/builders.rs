use congru::{
    Adaptive, Aligner, Annealed, Distance, Error, Guarded, Icp, Kernel, Local, Metric, Multistart,
    Search, Target,
};

use super::Model;

pub fn point() -> Icp {
    Icp {
        metric: Metric::PointToPoint,
        ..Icp::default()
    }
}

pub fn tukey(span: f64) -> Result<Icp, Error> {
    Ok(Icp {
        kernel: Kernel::Tukey(Distance::new(span / 30.0)?),
        max_correspondence_distance: Some(Distance::new(span / 10.0)?),
        ..Icp::default()
    })
}

pub fn trimmed(span: f64) -> Result<Icp, Error> {
    Ok(Icp {
        kernel: Kernel::Cauchy(Distance::new(span / 40.0)?),
        overlap: congru::Fraction::new(0.7)?,
        ..Icp::default()
    })
}

pub fn local(model: &Model, icp: Icp) -> Result<Box<dyn Aligner>, Error> {
    Ok(Box::new(Local {
        name: String::new(),
        icp,
        target: Target::from_oriented(model.target.clone())?,
    }))
}

pub fn annealed(model: &Model) -> Result<Box<dyn Aligner>, Error> {
    Ok(Box::new(Annealed {
        name: String::new(),
        icp: Icp::default(),
        coarsest: Distance::new(model.span / 5.0)?,
        finest: Distance::new(model.span / 80.0)?,
        target: Target::from_oriented(model.target.clone())?,
    }))
}

fn multistart(span: f64) -> Result<Multistart, Error> {
    Ok(Multistart {
        starts: 32,
        icp: Icp {
            kernel: Kernel::Cauchy(Distance::new(span / 40.0)?),
            ..Icp::default()
        },
    })
}

pub fn search(model: &Model) -> Result<Box<dyn Aligner>, Error> {
    Ok(Box::new(Search {
        name: String::new(),
        multistart: multistart(model.span)?,
        target: Target::from_oriented(model.target.clone())?,
    }))
}

pub fn guarded(model: &Model) -> Result<Box<dyn Aligner>, Error> {
    Ok(Box::new(Guarded {
        name: String::new(),
        local: tukey(model.span)?,
        search: multistart(model.span)?,
        min_inliers: 0.8,
        target: Target::from_oriented(model.target.clone())?,
    }))
}

pub fn adaptive(model: &Model) -> Result<Box<dyn Aligner>, Error> {
    Ok(Box::new(Adaptive {
        name: String::new(),
        icp: Icp::default(),
        first_deviation: Distance::new(model.span / 20.0)?,
        stages: 4,
        target: Target::from_oriented(model.target.clone())?,
    }))
}
