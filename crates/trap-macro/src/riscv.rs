macro_rules! save {
    ($reg:ident => $ptr:ident[$pos:expr]) => {
        concat!(
            "sd ",
            stringify!($reg),
            ", 8*",
            $pos,
            '(',
            stringify!($ptr),
            ')'
        )
    };
}

macro_rules! load {
    ($ptr:ident[$pos:expr] => $reg:ident) => {
        concat!(
            "ld ",
            stringify!($reg),
            ", 8*",
            $pos,
            '(',
            stringify!($ptr),
            ')'
        )
    };
}

macro_rules! concat_with_newline {
    ($($e:expr),* $(,)?) => {
        concat!($(concat!($e, "\n"),)*)
    };
}

pub const KERNEL_TRAP: &'static str = // 在内核栈上分配保存上下文的空间，
    concat_with_newline!(
        "addi sp, sp, -{size}",
        // 保存调用者保存的寄存器
        save!(ra => sp[0]),
        save!(t0 => sp[1]),
        save!(t1 => sp[2]),
        save!(t2 => sp[3]),
        save!(t3 => sp[4]),
        save!(t4 => sp[5]),
        save!(t5 => sp[6]),
        save!(t6 => sp[7]),
        save!(a0 => sp[8]),
        save!(a1 => sp[9]),
        save!(a2 => sp[10]),
        save!(a3 => sp[11]),
        save!(a4 => sp[12]),
        save!(a5 => sp[13]),
        save!(a6 => sp[14]),
        save!(a7 => sp[15]),
        // 将指向保存上下文空间的指针传入第一个参数
        "mv a0, sp",
        "jal {handler}",
        // 恢复调用者保存的寄存器
        load!(sp[ 0] => ra),
        load!(sp[ 1] => t0),
        load!(sp[ 2] => t1),
        load!(sp[ 3] => t2),
        load!(sp[ 4] => t3),
        load!(sp[ 5] => t4),
        load!(sp[ 6] => t5),
        load!(sp[ 7] => t6),
        load!(sp[ 8] => a0),
        load!(sp[ 9] => a1),
        load!(sp[10] => a2),
        load!(sp[11] => a3),
        load!(sp[12] => a4),
        load!(sp[13] => a5),
        load!(sp[14] => a6),
        load!(sp[15] => a7),
        "addi sp, sp, {size}",
        "sret"
    );
