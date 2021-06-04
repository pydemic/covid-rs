use std::iter;

use crate::prelude::{Real, INF};

/// ASCII plot of a sequence of positive values.
///
/// Draw each point as a column filled with '*'s up to the maximum height.
pub fn plot_vbars(value: &[Real], height: usize) {
    if value.is_empty() {
        return;
    }
    let max = value.iter().cloned().fold(-INF, |x, y| x.max(y));
    let step = max / height as Real;

    for i in 0..height + 1 {
        let mut ln = String::with_capacity(value.len());
        let h = (height - i) as Real * step;
        for &x in value {
            ln.push(if x >= h { '*' } else { ' ' });
        }
        println!("{}", ln);
    }
}

/// ASCII plot of a sequence of positive values horizontally.
///
/// Draw each is a row filled with '*'s up to the maximum width.
pub fn plot_hbars(values: &[Real], width: usize) {
    if values.is_empty() {
        return;
    }
    let max = values.iter().cloned().fold(-INF, |x, y| x.max(y));
    let step = max / width as Real;

    for &x in values {
        let n = (x / step) as usize;
        let mut ln = String::with_capacity(n + 1);
        ln.push('|');
        ln.extend(iter::repeat('=').take(n));
        println!("{}", ln);
    }
}
