// This code was written while referencing https://github.com/tommz9/pylinerating
// Credit to Tomas Barton for initial calculation methods. (Thanks so much!)

pub struct Conductor {
    pub stranded: bool,
    pub high_rs: bool,
    pub diameter: f64,
    pub cross_section: f64,
    pub absorptivity: f64,
    pub emissivity: f64,
    pub r_at_25: f64,
    pub r_at_75: f64,
}

pub struct HeatMaterial {
    pub name: &'static str,
    pub conductivity: f64,
    pub specific_heat: f64,
    pub thermal_expansion: f64,
}

pub const MATERIALS_HEAT: [HeatMaterial; 2] = [
    HeatMaterial {
        name: "steel",
        conductivity: 0.5119,
        specific_heat: 481.0,
        thermal_expansion: 1.00e-4,
    },
    HeatMaterial {
        name: "aluminum",
        conductivity: 1.116,
        specific_heat: 897.0,
        thermal_expansion: 3.80e-4,
    },
];

pub fn adjust_r(conductor_temperature: f64, r_at_25: f64, r_at_75: f64) -> f64 {
    let per_1: f64 = (r_at_75 - r_at_25) / (75.0 - 25.0);
    r_at_25 + ((conductor_temperature - 25.0) * per_1)
}

pub fn dynamic_viscosity(ambient_temperature: f64, conductor_temperature: f64) -> f64 {
    let tfilm = (conductor_temperature + ambient_temperature) / 2.0;
    1.458e-6 * (tfilm + 273.0).powf(1.5) / (tfilm + 383.4)
}

pub fn air_density(ambient_temperature: f64, conductor_temperature: f64, elevation: f64) -> f64 {
    let tfilm = (conductor_temperature + ambient_temperature) / 2.0;
    (1.293 - 1.525e-4 * elevation + 6.379e-9 * elevation.powi(2)) / (1.0 + 0.00367 * tfilm)
}

pub fn thermal_conductivity_of_air(ambient_temperature: f64, conductor_temperature: f64) -> f64 {
    let tfilm = (conductor_temperature + ambient_temperature) / 2.0;
    2.424e-2 + 7.477e-5 * tfilm - 4.407e-9 * tfilm.powi(2)
}

pub fn reynolds_number(
    ambient_temperature: f64,
    wind_speed: f64,
    conductor: &Conductor,
    conductor_temperature: f64,
    elevation: f64,
) -> f64 {
    conductor.diameter
        * air_density(ambient_temperature, conductor_temperature, elevation)
        * wind_speed
        / dynamic_viscosity(ambient_temperature, conductor_temperature)
}

pub fn forced_convection(
    ambient_temperature: f64,
    wind_speed: f64,
    angle_of_attack: f64,
    conductor: &Conductor,
    conductor_temperature: f64,
    elevation: f64,
) -> (f64, Option<(f64, f64, f64, f64)>) {
    let kangle = 1.194
        - angle_of_attack.cos()
        + 0.194 * (2.0 * angle_of_attack).cos()
        + 0.368 * (2.0 * angle_of_attack).sin();

    let nre = reynolds_number(
        ambient_temperature, wind_speed, &conductor, conductor_temperature, elevation,
    );

    let kf = thermal_conductivity_of_air(ambient_temperature, conductor_temperature);

    let qc1 = kangle
        * (1.01 + 1.35 * nre.powf(0.52))
        * kf
        * (conductor_temperature - ambient_temperature);

    let qc2 = kangle
        * 0.754 * nre.powf(0.6)
        * kf
        * (conductor_temperature - ambient_temperature);

    (f64::max(qc1, qc2), Some((qc1, qc2, kangle, nre)))
}

pub fn natural_convection(
    ambient_temperature: f64,
    conductor: &Conductor,
    conductor_temperature: f64,
    elevation: f64,
) -> f64 {
    3.645
        * air_density(ambient_temperature, conductor_temperature, elevation).sqrt()
        * conductor.diameter.powf(0.75)
        * (conductor_temperature - ambient_temperature).powf(1.25)
}

pub fn convective_heat_loss(
    ambient_temperature: f64,
    wind_speed: f64,
    angle_of_attack: f64,
    conductor: &Conductor,
    conductor_temperature: f64,
    elevation: f64,
) -> f64 {
    let forced: (f64, Option<(f64, f64, f64, f64)>) = forced_convection(
        ambient_temperature,
        wind_speed,
        angle_of_attack,
        &conductor,
        conductor_temperature,
        elevation,
    );

    let natural = natural_convection(
        ambient_temperature, &conductor, conductor_temperature, elevation
    );

    f64::max(forced.0, natural)
}

pub fn radiated_heat_loss(
    ambient_temperature: f64,
    conductor: &Conductor,
    conductor_temperature: f64,
) -> f64 {
    17.8
        * conductor.diameter
        * conductor.emissivity
        * (
            ((conductor_temperature + 273.0) / 100.0).powi(4)
            - ((ambient_temperature + 273.0) / 100.0).powi(4)
        )
}

pub fn solar_heat_gain(solar_irradiation: f64, conductor: &Conductor) -> f64 {
    conductor.absorptivity * solar_irradiation * conductor.diameter
}

pub fn thermal_rating(
    ambient_temperature: f64,
    wind_speed: f64,
    angle_of_attack: f64,
    solar_irradiation: f64,
    conductor: &Conductor,
    conductor_temperature: f64,
    horizontal_angle: f64,
    elevation: f64,
) -> f64 {
    // the angle must be in the range 0-90
    let angle_of_attack = 90.0 - ((angle_of_attack % 180.0) - 90.0).abs();
    let angle_of_attack = (angle_of_attack / 180.0) * std::f64::consts::PI;

    let qc = convective_heat_loss(
        ambient_temperature,
        wind_speed,
        angle_of_attack,
        &conductor,
        conductor_temperature,
        elevation,
    );

    let qr = radiated_heat_loss(ambient_temperature, &conductor, conductor_temperature);

    let qs = solar_heat_gain(solar_irradiation, &conductor);

    let current = ((qc + qr - qs) / adjust_r(conductor_temperature, conductor.r_at_25, conductor.r_at_75)).sqrt();

    current
}

