use anyhow::Result;
use std::{convert::TryFrom, fs};
use tch::{nn::Init, nn::VarStore, Device, Kind};

#[test]
fn var_store_entry() {
    let vs = VarStore::new(Device::Cpu);
    let root = vs.root();

    let t1 = root.entry("key").or_zeros(&[3, 1, 4]);
    let t2 = root.entry("key").or_zeros(&[1, 5, 9]);

    assert_eq!(t1.size(), &[3, 1, 4]);
    assert_eq!(t2.size(), &[3, 1, 4]);
}

#[test]
fn save_and_load_var_store() -> Result<()> {
    let filename =
        std::env::temp_dir().join(format!("tch-vs-load-complete-{}", std::process::id()));
    let add = |vs: &tch::nn::Path| {
        let v = vs.sub("a").sub("b").ones("t2", &[3]);
        let u = vs.zeros("t1", &[4]);
        let _w = vs.sub("a").sub("b").sub("ccc").ones("t123", &[3]);
        let _w = vs.sub("a").sub("b").sub("ccc").ones("t123", &[3]);
        (u, v)
    };
    let vs1 = VarStore::new(Device::Cpu);
    let mut vs2 = VarStore::new(Device::Cpu);
    let (mut u1, mut v1) = add(&vs1.root());
    let (u2, v2) = add(&vs2.root());
    tch::no_grad(|| {
        u1 += 42.0;
        v1 *= 2.0;
    });
    assert_eq!(f32::try_from(&u1.mean(Kind::Float))?, 42.0);
    assert_eq!(f32::try_from(&v1.mean(Kind::Float))?, 2.0);
    assert_eq!(f32::try_from(&u2.mean(Kind::Float))?, 0.0);
    assert_eq!(f32::try_from(&v2.mean(Kind::Float))?, 1.0);
    vs1.save(&filename).unwrap();
    vs2.load(&filename).unwrap();
    assert_eq!(f32::try_from(&u1.mean(Kind::Float))?, 42.0);
    assert_eq!(f32::try_from(&u2.mean(Kind::Float))?, 42.0);
    assert_eq!(f32::try_from(&v2.mean(Kind::Float))?, 2.0);
    fs::remove_file(filename).unwrap();
    Ok(())
}

#[test]
fn save_and_load_partial_var_store() -> Result<()> {
    let filename = std::env::temp_dir().join(format!(
        "tch-vs-partial-load-complete-{}",
        std::process::id()
    ));
    let add = |vs: &tch::nn::Path| {
        let v = vs.sub("a").sub("b").ones("t2", &[3]);
        let u = vs.zeros("t1", &[4]);
        let _w = vs.sub("a").sub("b").sub("ccc").ones("t123", &[3]);
        let _w = vs.sub("a").sub("b").sub("ccc").ones("t123", &[3]);
        (u, v)
    };
    let vs1 = VarStore::new(Device::Cpu);
    let mut vs2 = VarStore::new(Device::Cpu);
    let (mut u1, mut v1) = add(&vs1.root());
    let (u2, v2) = add(&vs2.root());
    tch::no_grad(|| {
        u1 += 42.0;
        v1 *= 2.0;
    });
    assert_eq!(f32::try_from(&u1.mean(Kind::Float))?, 42.0);
    assert_eq!(f32::try_from(&v1.mean(Kind::Float))?, 2.0);
    assert_eq!(f32::try_from(&u2.mean(Kind::Float))?, 0.0);
    assert_eq!(f32::try_from(&v2.mean(Kind::Float))?, 1.0);
    vs1.save(&filename).unwrap();
    let missing_variables = vs2.load_partial(&filename).unwrap();
    assert_eq!(f32::try_from(&u1.mean(Kind::Float))?, 42.0);
    assert_eq!(f32::try_from(&u2.mean(Kind::Float))?, 42.0);
    assert_eq!(f32::try_from(&v2.mean(Kind::Float))?, 2.0);
    assert!(missing_variables.is_empty());
    fs::remove_file(filename).unwrap();
    Ok(())
}

#[test]
#[should_panic]
fn save_and_load_var_store_incomplete_file() {
    let filename =
        std::env::temp_dir().join(format!("tch-vs-load-incomplete-{}", std::process::id()));
    let add = |vs: &tch::nn::Path| {
        let u = vs.zeros("t1", &[4]);
        let _w = vs.sub("a").sub("b").sub("ccc").ones("t123", &[3]);
        u
    };
    let add_partial = |vs: &tch::nn::Path| {
        let v = vs.sub("a").sub("b").ones("t2", &[3]);
        let u = vs.zeros("t1", &[4]);
        let _w = vs.sub("a").sub("b").sub("ccc").ones("t123", &[3]);
        (u, v)
    };
    let vs1 = VarStore::new(Device::Cpu);
    let mut vs2 = VarStore::new(Device::Cpu);
    let mut u1 = add(&vs1.root());
    let (u2, v2) = add_partial(&vs2.root());
    tch::no_grad(|| {
        u1 += 42.0;
    });
    assert_eq!(f32::try_from(&u1.mean(Kind::Float)).unwrap(), 42.0);
    assert_eq!(f32::try_from(&u2.mean(Kind::Float)).unwrap(), 0.0);
    assert_eq!(f32::try_from(&v2.mean(Kind::Float)).unwrap(), 1.0);
    vs1.save(&filename).unwrap();
    vs2.load(&filename).unwrap();
    assert_eq!(f32::try_from(&u1.mean(Kind::Float)).unwrap(), 42.0);
    assert_eq!(f32::try_from(&u2.mean(Kind::Float)).unwrap(), 42.0);
    assert_eq!(f32::try_from(&v2.mean(Kind::Float)).unwrap(), 1.0);
    fs::remove_file(filename).unwrap();
}

#[test]
fn save_and_load_partial_var_store_incomplete_file() -> Result<()> {
    let filename = std::env::temp_dir().join(format!(
        "tch-vs-partial-load-incomplete-{}",
        std::process::id()
    ));
    let add = |vs: &tch::nn::Path| {
        let u = vs.zeros("t1", &[4]);
        let _w = vs.sub("a").sub("b").sub("ccc").ones("t123", &[3]);
        u
    };
    let add_partial = |vs: &tch::nn::Path| {
        let v = vs.sub("a").sub("b").ones("t2", &[3]);
        let u = vs.zeros("t1", &[4]);
        let _w = vs.sub("a").sub("b").sub("ccc").ones("t123", &[3]);
        (u, v)
    };
    let vs1 = VarStore::new(Device::Cpu);
    let mut vs2 = VarStore::new(Device::Cpu);
    let mut u1 = add(&vs1.root());
    let (u2, v2) = add_partial(&vs2.root());
    tch::no_grad(|| {
        u1 += 42.0;
    });
    assert_eq!(f32::try_from(&u1.mean(Kind::Float))?, 42.0);
    assert_eq!(f32::try_from(&u2.mean(Kind::Float))?, 0.0);
    assert_eq!(f32::try_from(&v2.mean(Kind::Float))?, 1.0);
    vs1.save(&filename).unwrap();
    let missing_variables = vs2.load_partial(&filename).unwrap();
    assert_eq!(f32::try_from(&u1.mean(Kind::Float))?, 42.0);
    assert_eq!(f32::try_from(&u2.mean(Kind::Float))?, 42.0);
    assert_eq!(f32::try_from(&v2.mean(Kind::Float))?, 1.0);
    assert_eq!(missing_variables, vec!(String::from("a.b.t2")));
    fs::remove_file(filename).unwrap();
    Ok(())
}

#[test]
fn init_test() -> Result<()> {
    let vs = VarStore::new(Device::Cpu);
    let zeros = vs.root().zeros("t1", &[3]);
    assert_eq!(Vec::<f32>::try_from(&zeros)?, [0., 0., 0.]);
    let zeros = vs.root().var("t2", &[3], Init::Const(0.));
    assert_eq!(Vec::<f32>::try_from(&zeros)?, [0., 0., 0.]);
    let ones = vs.root().var("t3", &[3], Init::Const(1.));
    assert_eq!(Vec::<f32>::try_from(&ones)?, [1., 1., 1.]);
    let ones = vs.root().var("t4", &[3], Init::Const(0.5));
    assert_eq!(Vec::<f32>::try_from(&ones)?, [0.5, 0.5, 0.5]);
    let forty_two = vs.root().var("t4", &[2], Init::Const(42.));
    assert_eq!(Vec::<f32>::try_from(&forty_two)?, [42., 42.]);
    let uniform = vs
        .root()
        .var("t5", &[100], Init::Uniform { lo: 1.0, up: 2.0 });
    let uniform_min = f32::try_from(&uniform.min())?;
    let uniform_max = f32::try_from(&uniform.max())?;
    assert!(uniform_min >= 1., "min {}", uniform_min);
    assert!(uniform_max <= 2., "max {}", uniform_max);
    let uniform_std = f32::try_from(&uniform.std(true))?;
    assert!(
        uniform_std > 0.15 && uniform_std < 0.35,
        "std {}",
        uniform_std
    );
    let normal = vs.root().var(
        "normal",
        &[100],
        Init::Randn {
            mean: 0.,
            stdev: 0.02,
        },
    );
    let normal_std = f32::try_from(&normal.std(true))?;
    assert!(normal_std <= 0.03, "std {}", normal_std);
    let mut vs2 = VarStore::new(Device::Cpu);
    let ones = vs2.root().ones("t1", &[3]);
    assert_eq!(Vec::<f32>::try_from(&ones)?, [1., 1., 1.]);
    vs2.copy(&vs).unwrap();
    assert_eq!(Vec::<f32>::try_from(&ones)?, [0., 0., 0.]);
    Ok(())
}
