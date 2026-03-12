pub trait RhythmElement {
    fn tick() -> Self;
    fn mute() -> Self;
}

impl<T : From<bool>> RhythmElement for T {
    fn tick() -> Self {
        true.into()
    }
    fn mute() -> Self {
        false.into()
    }
}

pub fn euclid_from_fn<T, F : Fn() -> T>(
    k : usize, 
    n : usize,
    r : usize,
    tick_fn : F,
    mute_fn : F
)  -> Vec<T> {
    let mut res : Vec<T> = (0..n).map(|_| mute_fn()).collect();
    
    if n % k == 0 {
        for i in 0..k {
            res[i * (n / k)] = tick_fn();
        }
        return res;
    }

    let init_rem = std::cmp::min(n - k, k);

    let mut lines : Vec<Vec<T>> = vec![
        (0..(n - init_rem)).map(|_| mute_fn()).collect(),
        (0..init_rem).map(|_| mute_fn()).collect()
    ];
    for i in 0..k {
        lines[0][i] = tick_fn();
    }
    let mut last_line_len = lines.last().unwrap().len();
    let mut rem = lines[0].len() % last_line_len;
    while rem > 1 {
        let n_lines = lines.len();
        for l_i in 0..n_lines {
            let line_len =  lines[l_i].len();
            let rem_line = line_len % last_line_len;
            if rem_line > 0 {
                let end = lines[l_i].split_off(line_len - rem_line);
                lines.push(end);
            }
        }
        last_line_len = lines.last().unwrap().len();
        rem = lines[0].len() % last_line_len;
    }
    
    let mut line = 0;
    let mut col = 0;
    for _ in 0..r {
        line += 1;
        if line >= lines.len() || lines[line].len() <= col {
            line = 0;
            col = (col + 1) % lines[0].len();
        }
    }
    for i in 0..n {
        res[i] = std::mem::replace(&mut lines[line][col], mute_fn());
        line += 1;
        if line >= lines.len() || lines[line].len() <= col {
            line = 0;
            col = (col + 1) % lines[0].len();
        }
    }
    res
}

pub fn euclid<T : RhythmElement>(k : usize, n : usize, r : usize) -> Vec<T> {
    euclid_from_fn(k, n, r, T::tick as fn() -> T, T::mute as fn() -> T)
}

pub fn bitrhythm_from_fn<T, F : Fn() -> T>(
    mut i : u64, 
    tick_fn : F,
    mute_fn : F
) -> Vec<T> {
    let mut res = Vec::new();

    while i > 0 {
        if i % 2 == 1 {
            res.push(tick_fn());
        } else {
            res.push(mute_fn());
        }
        i >>= 1;
    }
    
    res.reverse();
    res
}

pub fn bitrhythm<T : RhythmElement>(i : u64) -> Vec<T> {
    bitrhythm_from_fn(i, T::tick as fn() -> T, T::mute as fn() -> T)
}