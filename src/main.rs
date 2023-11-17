// mod ieee738;
mod ieee738_us;

fn main() {
    println!("Hello, world!");
    // Sample input values
    // let ambient_temperature = 50.0;
    // let wind_speed = 0.61;
    // let angle_of_attack = 90.0;
    // let solar_irradiation = 1000.0;
    // let conductor_temperature = 100.0;
    // let horizontal_angle = 0.0;
    // let elevation = 0.0;

    // let conductor = IEEE738::Conductor {
    //     stranded: true,
    //     high_rs: true,
    //     diameter: 28.14e-3,
    //     cross_section: 0.0,
    //     absorptivity: 0.8,
    //     emissivity: 0.8,
    //     r_at_25: 7.283e-5,
    //     r_at_75: 8.688e-5,
    // };

    // let rating = IEEE738::thermal_rating(
    //     ambient_temperature,
    //     wind_speed,
    //     angle_of_attack,
    //     solar_irradiation,
    //     &conductor,
    //     conductor_temperature,
    //     horizontal_angle,
    //     elevation
    // );

    // println!("Thermal Rating: {}", rating);

    let solar_radiation: f64 = -1.0; // 94.6
    let month: i32 = 6;
    let day_of_month: i32 = 10;
    let hour_of_day: f64 = 11.0;
    let ambient_temperature: f64 = 40.0;
    let wind_speed: f64 = 2.0;
    let wind_angle_deg: f64 = 90.0;
    let latitude_deg: f64 = 30.0;
    let line_azimuth_deg: f64 = 90.0;
    let elevation: f64 = 0.0;
    let atmosphere_clear: bool = true;
    let conductor_temperature: f64 = 100.0;
    let absorptivity: f64 = 0.8;
    let emissivity: f64 = 0.8;
    let diameter: f64 = 0.092333333;
    let t_low: f64 = 25.0;
    let t_high: f64 = 75.0;
    let r_low: f64 = 2.20833e-05;
    let r_high: f64 = 2.63258e-05;

    let rating = ieee738_us::thermal_rating(
        solar_radiation
        ,month
        ,day_of_month
        ,hour_of_day
        ,ambient_temperature
        ,wind_speed
        ,wind_angle_deg
        ,latitude_deg
        ,line_azimuth_deg
        ,elevation
        ,atmosphere_clear
        ,conductor_temperature
        ,absorptivity
        ,emissivity
        ,diameter
        ,t_low
        ,t_high
        ,r_low
        ,r_high
    );

    println!("Thermal Rating: {}", rating);

    let tolerance: f64 = 0.01;
    let temperature = ieee738_us::calculated_temperature(
        solar_radiation
        ,month
        ,day_of_month
        ,hour_of_day
        ,ambient_temperature
        ,wind_speed
        ,wind_angle_deg
        ,latitude_deg
        ,line_azimuth_deg
        ,elevation
        ,atmosphere_clear
        ,rating
        ,tolerance
        ,absorptivity
        ,emissivity
        ,diameter
        ,t_low
        ,t_high
        ,r_low
        ,r_high
    );

    println!("Temperature: {}", temperature);

    let heat_capacity: f64 = 305.6328;
    let final_current: f64 = 2000.0;
    let time_step: f64 = 60.0;
    let steps: i32 = 1;

    let delta_t = ieee738_us::conductor_temperature_rise(
        solar_radiation
        ,month
        ,day_of_month
        ,hour_of_day
        ,ambient_temperature
        ,wind_speed
        ,wind_angle_deg
        ,latitude_deg
        ,line_azimuth_deg
        ,elevation
        ,atmosphere_clear
        ,conductor_temperature
        ,final_current
        ,time_step
        ,steps
        ,absorptivity
        ,emissivity
        ,diameter
        ,t_low
        ,t_high
        ,r_low
        ,r_high
        ,heat_capacity
    );

    println!("Delta T: {}", delta_t);

    let conductor_temperature_max: f64 = 254.3;

    let steps: i32 = 31;

    let t_rating: f64 = ieee738_us::transient_rating(
        solar_radiation
        ,month
        ,day_of_month
        ,hour_of_day
        ,ambient_temperature
        ,wind_speed
        ,wind_angle_deg
        ,latitude_deg
        ,line_azimuth_deg
        ,elevation
        ,atmosphere_clear
        ,conductor_temperature
        ,conductor_temperature_max
        ,time_step
        ,steps
        ,tolerance
        ,absorptivity
        ,emissivity
        ,diameter
        ,t_low
        ,t_high
        ,r_low
        ,r_high
        ,heat_capacity
    );

    println!("Transient Rating: {}", t_rating);

}

