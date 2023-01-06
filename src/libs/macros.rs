#[macro_export]
macro_rules! _overloading_block {
    ($ops:ident, $fn_name:ident, $r_type:ty, $l_type:ty, $res_type:ty, $l_var:ident, $r_var:ident, $blck:block) => {
        impl std::ops::$ops<$r_type> for $l_type {
            type Output = $res_type;

            fn $fn_name(self, $r_var: $r_type) -> Self::Output {
                let $l_var = self;
                $blck
            }
        }
    };
}

#[macro_export]
macro_rules! _trait_block {
    ($ops:ident, $fn_name:ident, $r_type:ty, $l_type:ty, $res_type:ty, $l_var:ident, $r_var:ident, $blck:block) => {
        impl $ops<$r_type> for $l_type {
            type Output = $res_type;

            fn $fn_name(self, $r_var: $r_type) -> Self::Output {
                let $l_var = self;
                $blck
            }
        }
    };
}

#[macro_export]
macro_rules! _overloading_core {
    (+, $($t:tt)+) => {
        _overloading_block!(Add, add, $($t)+);
    };
    (-, $($t:tt)+) => {
        _overloading_block!(Sub, sub, $($t)+);
    };
    (*, $($t:tt)+) => {
        _overloading_block!(Mul, mul, $($t)+);
    };
    (/, $($t:tt)+) => {
        _overloading_block!(Div, div, $($t)+);
    };
}

#[macro_export]
macro_rules! _trait_core {
    ($ops:ident, $fn_name:ident, $($t:tt)+) => {
        _trait_block($ops, $fn_name, $($t:tt)+)
    };
}

#[macro_export]
macro_rules! overloading {
    (($l_var:ident : $l_type:ty) $ops:tt ($r_var:ident : $r_type:ty) => $res_type:ty as $blck:block) => {
        _overloading_core!($ops, $r_type, $l_type, $res_type, $l_var, $r_var, $blck);
        _overloading_core!($ops, $r_type, &$l_type, $res_type, $l_var, $r_var, $blck);
        _overloading_core!($ops, &$r_type, $l_type, $res_type, $l_var, $r_var, $blck);
        _overloading_core!($ops, &$r_type, &$l_type, $res_type, $l_var, $r_var, $blck);
    };
    (^($l_var:ident : $l_type:ty) $ops:tt ($r_var:ident : $r_type:ty) => $res_type:ty as $blck:block) => {
        _overloading_core!($ops, $r_type, $l_type, $res_type, $l_var, $r_var, $blck);
        _overloading_core!($ops, &$r_type, $l_type, $res_type, $l_var, $r_var, $blck);
    };
    (($l_var:ident : $l_type:ty) $ops:tt ^($r_var:ident : $r_type:ty) => $res_type:ty as $blck:block) => {
        _overloading_core!($ops, $r_type, $l_type, $res_type, $l_var, $r_var, $blck);
        _overloading_core!($ops, $r_type, &$l_type, $res_type, $l_var, $r_var, $blck);
    };
    (($ops:ident) ($l_var:ident : $l_type:ty) ($fn_name:ident) ($r_var:ident : $r_type:ty) => $res_type:ty as $blck:block) => {
        _trait_block!($ops, $fn_name, $r_type, $l_type, $res_type, $l_var, $r_var, $blck);
        _trait_block!($ops, $fn_name, &$r_type, $l_type, $res_type, $l_var, $r_var, $blck);
        _trait_block!($ops, $fn_name, $r_type, &$l_type, $res_type, $l_var, $r_var, $blck);
        _trait_block!($ops, $fn_name, &$r_type, &$l_type, $res_type, $l_var, $r_var, $blck);
    };
}

#[macro_export]
macro_rules! _lhs_rhs_prime_eq_check {
    ($lhs:ident, $rhs:ident) => {
        if $lhs.prime != $rhs.prime {
            panic!(
                "[FiniteField] lhs's prime {} != rhs's prime {}",
                $lhs.prime, $rhs.prime
            );
        }
    };
}
