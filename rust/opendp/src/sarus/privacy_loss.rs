use std::{convert::TryFrom, ops::Bound, rc::Rc};

use rug::{Float, float::Round, Rational};

use crate::{core::{Domain, Function, Measure, Measurement, Metric, PrivacyRelation}, dist::SymmetricDistance, dom::{IntervalDomain, PairDomain}, error::Fallible, meas::{GaussianDomain, LaplaceDomain}};

use super::PLDistribution;

const PREC:u32 = 128;
const GRID_SIZE:usize = 100;

/// A Measure that comes with a privacy loss distribution.
#[derive(Clone)]
pub struct PLDSmoothedMaxDivergence<MI> where MI: Metric {
    privacy_loss_distribution: Rc<dyn Fn(&MI::Distance) -> Fallible<PLDistribution>>,
}

impl<MI> PLDSmoothedMaxDivergence<MI> where MI: Metric {
    pub fn new(privacy_loss_distribution: Rc<dyn Fn(&MI::Distance) -> Fallible<PLDistribution>>) -> Self {
        PLDSmoothedMaxDivergence {
            privacy_loss_distribution: privacy_loss_distribution
        }
    }

    pub fn f(&self, d_in: &MI::Distance) -> Vec<(f64, f64)> {
        (self.privacy_loss_distribution)(d_in).unwrap_or_default().f()
    }

    pub fn simplified_f(&self, d_in: &MI::Distance) -> Vec<(f64, f64)> {
        (self.privacy_loss_distribution)(d_in).unwrap_or_default().simplified().f()
    }
}

impl<MI> Default for PLDSmoothedMaxDivergence<MI> where MI: Metric {
    fn default() -> Self {
        PLDSmoothedMaxDivergence::new(Rc::new(|_:&MI::Distance| Ok(PLDistribution::default())))
    }
}

impl<MI> PartialEq for PLDSmoothedMaxDivergence<MI> where MI:Metric {
    fn eq(&self, _other: &Self) -> bool { true }
}

impl<MI> Measure for PLDSmoothedMaxDivergence<MI> where MI: Metric, MI::Distance: Clone {
    type Distance = (f64, f64);
}

// A way to build privacy relations from privacy loss distribution approximation

pub fn make_pld_privacy_relation<MI>(privacy_loss_distribution: Rc<dyn Fn(&MI::Distance) -> Fallible<PLDistribution>>) -> PrivacyRelation<MI, PLDSmoothedMaxDivergence<MI>>
    where MI: Metric, MI::Distance: 'static + Clone {
    PrivacyRelation::new_fallible( move |d_in: &MI::Distance, (epsilon, delta): &(f64, f64)| {
        if delta<=&0.0 {
            return fallible!(InvalidDistance, "Privacy Loss Mechanism: delta must be positive")
        }
        let mut exp_epsilon = rug::Float::with_val_round(64, epsilon, Round::Down).0;
        exp_epsilon.exp_round(Round::Down);
        Ok(delta >= &privacy_loss_distribution(d_in)?.delta(&Rational::try_from(exp_epsilon).unwrap_or_default()))
    })
}

// Gaussian mechanism
fn gaussian_cdf(x:Float, mu:Float, sigma:Float) -> Float {
    0.5*(1.0 + Float::erf((x-mu)/(Float::with_val(PREC,2.0).sqrt()*sigma)))
}

/// Gaussian pld
fn gaussian_pld<'a>(scale: f64, grid_size: usize) -> impl Fn(&f64) -> Fallible<PLDistribution> + 'a {
    let min = Float::with_val(PREC, -3.0)*scale.clone();
    let max = Float::with_val(PREC, 3.0)*scale.clone();
    let sigma = Float::with_val(PREC, scale);
    move |d_in| {
        let mu = Float::with_val(PREC, d_in);
        let mut last_x = Float::with_val(PREC, min.clone());
        let mut x = Float::with_val(PREC, min.clone());
        let mut exp_eps = Float::with_val(PREC, 0);
        let mut prob = gaussian_cdf(x.clone(),Float::with_val(PREC, 0),sigma.clone());
        let mut exp_privacy_loss_probabilities = Vec::<(Float, Float)>::new();
        exp_privacy_loss_probabilities.push((
            exp_eps,
            prob
        ));
        for k in 0..grid_size {
            x = min.clone() + (max.clone()-min.clone())*Float::with_val(PREC, k+1)/Float::with_val(PREC, grid_size);
            exp_eps = Float::exp((mu.clone()*x.clone()-0.5*mu.clone().square())/sigma.clone().square());
            prob = gaussian_cdf(x.clone(),Float::with_val(PREC, 0),sigma.clone()) - gaussian_cdf(last_x.clone(),Float::with_val(PREC, 0),sigma.clone());
            exp_privacy_loss_probabilities.push((
                exp_eps.clone(),
                prob.clone(),
            ));
            last_x = x;
        }
        Ok(PLDistribution::from(exp_privacy_loss_probabilities))
    }
}

pub fn make_pld_gaussian<D>(scale: f64) -> Fallible<Measurement<D, D, D::Metric, PLDSmoothedMaxDivergence<D::Metric>>>
    where D: GaussianDomain<Atom=f64> {
    if scale<0.0 {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    let privacy_loss_distribution = Rc::new(gaussian_pld(scale.clone(), GRID_SIZE));
    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale.clone()),
        D::Metric::default(),
        PLDSmoothedMaxDivergence::new(privacy_loss_distribution.clone()),
        make_pld_privacy_relation(privacy_loss_distribution),
    ))
}

fn laplace_cdf(x:Float, mu:Float, sigma:Float) -> Float {
    let y = (x.clone()-mu.clone())/sigma.clone();
    Float::with_val(PREC, 0.5)*(
        Float::with_val(PREC, 1)+y.clone().signum()*(
            Float::with_val(PREC, 1)-Float::exp(-(y.clone().abs()))
        )
    )
}

// fn laplace_pdf(x:Float, mu:Float, sigma:Float) -> Float {
//     let y = (x.clone()-mu.clone())/sigma.clone();
//     Float::exp(-(y.clone().abs()))/sigma.clone()
// }

fn laplace_exp_eps(x:Float, mu:Float, sigma:Float) -> Float {
    if x.clone()-&mu< Float::with_val(PREC, -5)*&sigma {
        Float::with_val(PREC, 0)
    } else if x.clone()-&mu > Float::with_val(PREC, 5)*&sigma {
        Float::with_val(PREC, 1)
    } else {
        Float::exp((x.clone()/sigma.clone()).abs()-((x.clone()-mu.clone())/sigma.clone()).abs())
    }
}

/// Laplace pld
fn laplace_pld<'a>(scale: f64, grid_size: usize) -> impl Fn(&f64) -> Fallible<PLDistribution> + 'a {
    let min = Float::with_val(PREC, -5.0)*scale.clone();
    let max = Float::with_val(PREC, 5.0)*scale.clone();
    let sigma = Float::with_val(PREC, scale);
    move |d_in| {
        let mu = Float::with_val(PREC, d_in);
        let mut last_x = Float::with_val(PREC, min.clone());
        let mut x = Float::with_val(PREC, min.clone());
        let mut exp_eps = Float::exp(-mu.clone()/sigma.clone());
        let mut prob = laplace_cdf(x.clone(),Float::with_val(PREC, 0),sigma.clone());
        let mut exp_privacy_loss_probabilities = Vec::<(Float, Float)>::new();
        exp_privacy_loss_probabilities.push((
            exp_eps,
            prob
        ));
        for k in 0..grid_size {
            x = min.clone() + (max.clone()-min.clone())*Float::with_val(PREC, k+1)/Float::with_val(PREC, grid_size);
            exp_eps = laplace_exp_eps(x.clone(),mu.clone(),sigma.clone());
            prob = laplace_cdf(x.clone(),Float::with_val(PREC, 0),sigma.clone()) - laplace_cdf(last_x.clone(),Float::with_val(PREC, 0),sigma.clone());
            // prob = laplace_pdf(x.clone(),Float::with_val(PREC, 0),sigma.clone());
            exp_privacy_loss_probabilities.push((
                exp_eps.clone(),
                prob.clone(),
            ));
            last_x = x;
        }
        Ok(PLDistribution::from(exp_privacy_loss_probabilities))
    }
}

pub fn make_pld_laplace<D>(scale: f64) -> Fallible<Measurement<D, D, D::Metric, PLDSmoothedMaxDivergence<D::Metric>>>
    where D: LaplaceDomain<Atom=f64> {
    if scale<0.0 {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    let privacy_loss_distribution = Rc::new(laplace_pld(scale.clone(), GRID_SIZE));
    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale.clone()),
        D::Metric::default(),
        PLDSmoothedMaxDivergence::new(privacy_loss_distribution.clone()),
        make_pld_privacy_relation(privacy_loss_distribution),
    ))
}

/// Randmized Response
fn epsilon_delta_pld<'a>(epsilon:f64, delta:f64) -> impl Fn(&u32) -> Fallible<PLDistribution> + 'a {
    move |n| {
        let n_exp_eps = ((*n as f64)*epsilon).exp();
        let n_delta = (*n as f64)*delta;
        Ok(PLDistribution::from(vec![(0.0, n_delta), (n_exp_eps, (1.0-n_delta)/(1.0+n_exp_eps)), (n_exp_eps.recip(), (1.0-n_delta)*n_exp_eps/(1.0+n_exp_eps))]))
    }
}

pub fn make_pld_epsilon_delta(epsilon:f64, delta:f64) -> Fallible<Measurement<IntervalDomain<u32>, IntervalDomain<u32>, SymmetricDistance, PLDSmoothedMaxDivergence<SymmetricDistance>>> {
    if delta<0.0 {
        return fallible!(MakeMeasurement, "delta must not be negative")
    }
    let input_domain = IntervalDomain::new(Bound::Included(0u32), Bound::Excluded(2u32)).unwrap();
    let output_domain = IntervalDomain::new(Bound::Included(0u32), Bound::Excluded(4u32)).unwrap();
    let privacy_loss_distribution = Rc::new(epsilon_delta_pld(epsilon, delta));
    Ok(Measurement::new(
        input_domain,
        output_domain,
        Function::new_fallible(|&_| fallible!(NotImplemented, "not implemented")),
        SymmetricDistance::default(),
        PLDSmoothedMaxDivergence::new(privacy_loss_distribution.clone()),
        make_pld_privacy_relation(privacy_loss_distribution),
    ))
}

pub fn make_pld_composition<DI, DO0, DO1, MI>(
    measurement0: &Measurement<DI, DO0, MI, PLDSmoothedMaxDivergence<MI>>,
    measurement1: &Measurement<DI, DO1, MI, PLDSmoothedMaxDivergence<MI>>)
    -> Fallible<Measurement<DI, PairDomain<DO0, DO1>, MI, PLDSmoothedMaxDivergence<MI>>>
    where DI: 'static + Domain,
          DO0: 'static + Domain,
          DO1: 'static + Domain,
          MI: 'static + Metric,
          MI::Distance: Clone {
    if measurement0.input_domain != measurement1.input_domain {
        return fallible!(DomainMismatch, "Input domain mismatch");
    } else if measurement0.input_metric != measurement1.input_metric {
        return fallible!(MetricMismatch, "Input metric mismatch");
    }
    let pld_0 = measurement0.output_measure.privacy_loss_distribution.clone();
    let pld_1 = measurement1.output_measure.privacy_loss_distribution.clone();
    let privacy_loss_distribution: Rc<dyn Fn(&MI::Distance) -> Fallible<PLDistribution>> = Rc::new(move |d_in:&MI::Distance| {
        Ok(&(pld_0)(d_in)? * &(pld_1)(d_in)?)
    });
    Ok(Measurement::new(
        measurement0.input_domain.clone(),
        PairDomain::new(measurement0.output_domain.clone(), measurement1.output_domain.clone()),
        Function::make_basic_composition(&measurement0.function, &measurement1.function),
        measurement0.input_metric.clone(),
        PLDSmoothedMaxDivergence::new(privacy_loss_distribution.clone()),
        make_pld_privacy_relation(privacy_loss_distribution),
    ))
}