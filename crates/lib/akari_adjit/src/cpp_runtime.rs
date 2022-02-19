use std::fmt::Write;
fn generate_vec_lib_(ty: String, n: usize, align: usize) -> String {
    let mut s = String::new();
    let vt = format!("{}x{}", ty, n);
    writeln!(
        &mut s,
        "struct alignas({}) {} {{ {} a[{}];\nusing Self={};static constexpr int N = {};Self()=default;",
        align, vt, ty, n,vt,n
    )
    .unwrap();
    writeln!(
        &mut s,
        "Self(const {}& x) {{for(int i=0;i<N;i++)a[i] = x; }}",
        ty
    )
    .unwrap();
    macro_rules! gen_ops {
        ($op:tt) => {
            writeln!(
                &mut s,
                r#"Self operator {0} (const Self&rhs)const{{
    auto v = Self{{}};
    for(int i=0;i<N;i++){{ v[i] = a[i] {0} rhs.a[i]; }}
    return v;
}}"#,
                stringify!($op)
            )
            .unwrap();
        };
    }
    gen_ops!(+);
    gen_ops!(-);
    gen_ops!(*);
    gen_ops!(/);
    writeln!(&mut s, "}};").unwrap();
    s
}
fn generate_vec_lib() -> String {
    format!(
        "{}{}{}",
        generate_vec_lib_("f32".into(), 2, 8),
        generate_vec_lib_("f32".into(), 3, 16),
        generate_vec_lib_("f32".into(), 4, 16)
    )
}

fn generate_runtime_lib() -> String {
    let mut s = String::new();
    writeln!(
        &mut s,
        r"template<class T>const T& read(const void* p){{
    return *reinterpret_cast<const T*>(p);
}}"
    )
    .unwrap();
    writeln!(&mut s, "{}", generate_vec_lib()).unwrap();
    s
}
