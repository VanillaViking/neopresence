pub fn get_diff(old: &str, new: &str) -> (u32, u32) {
    let mut deletions = 0;
    let mut additions = 0;

    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let max = old_lines.len() + new_lines.len();
    let mut v = vec![-1; 2 * max + 1];
    v[max + 1] = 0;
    let mut trace: Vec<Vec<i32>> = Vec::new();

    let mut x: i32 = 0;
    let mut y: i32 = 0;
    
    // TODO: change to linear space version
    'shortestedit: for d in 0..=max {
        trace.push(v.clone());
        for k in ((-1 * d as i32)..=(d as i32)).step_by(2) {
            if k == (-1 * (d as i32)) || (k != d as i32 && v[(k + max as i32) as usize -1] < v[(k + max as i32) as usize +1]) {
                x = v[(k + max as i32) as usize + 1];
            } else {
                x = v[(k + max as i32) as usize - 1] + 1;
            }
            y = x - k;

            while (x as usize) < old_lines.len() && (y as usize) < new_lines.len() && old_lines[x as usize] == new_lines[y as usize] {
                x += 1;
                y += 1;
            }
            v[(k + max as i32) as usize] = x;

            if x as usize >= old_lines.len() && y as usize >= new_lines.len() {
                // trace.push(v.clone());
                break 'shortestedit;
            }
        }
    }
    
    let mut k = old_lines.len() as i32 - new_lines.len() as i32;

    for trace_idx in (1..(trace.len())).rev() {
        let v = &trace[trace_idx];
        if k == (-1 * (trace_idx as i32)) || (k != trace_idx as i32 && v[(k + max as i32) as usize -1] < v[(k + max as i32) as usize +1]) {
            k = k + 1;
            additions += 1;
        } else {
            k = k - 1;
            deletions += 1;
        }
    }

    (deletions, additions)
}
