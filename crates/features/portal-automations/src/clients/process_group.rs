pub fn kill_group(group: u32) {
    let Ok(group) = libc::pid_t::try_from(group) else {
        return;
    };
    if group <= 1 {
        return;
    }
    unsafe {
        libc::killpg(group, libc::SIGKILL);
    }
}

pub fn effective_user() -> u32 {
    unsafe { libc::geteuid() }
}

pub fn effective_groups() -> Vec<u32> {
    let mut groups = vec![unsafe { libc::getegid() }];
    let count = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    if count > 0 {
        let mut buffer: Vec<libc::gid_t> = vec![0; count as usize];
        let filled = unsafe { libc::getgroups(count, buffer.as_mut_ptr()) };
        if filled > 0 {
            buffer.truncate(filled as usize);
            groups.extend(buffer);
        }
    }
    groups
}
