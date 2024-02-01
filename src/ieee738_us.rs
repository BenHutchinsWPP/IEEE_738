

/// Returns convective_heat_loss Qc (Watts / ft)
/// # Arguments
/// * `ambient_temperature` - T_a: Degrees (C)
/// * `wind_speed` - V_w: Wind Speed (ft/s)
/// * `wind_angle_deg` - Wind Angle (Degrees) 0 to 90
/// * `elevation` - H_e: Height of conductor above sea level (ft)
/// * `conductor_temperature` - T_s: Conductor Surface Temperature (C)
/// * `diameter` - D_0: Outer diameter of the conductor (ft)
pub fn convective_heat_loss(
    ambient_temperature: f64,
    wind_speed: f64,
    wind_angle_deg: f64,
    elevation: f64,
    conductor_temperature: f64,
    diameter: f64,
) -> f64 {
    let pi = std::f64::consts::PI;

    // Limit to within 0-90.
    let wind_angle_deg_limited = 90.0 - (wind_angle_deg % 180.0 - 90.0).abs();
    let wind_angle_rad = wind_angle_deg_limited * (pi / 180.0);

    // Equation 6, Tfilm W/ft (degrees C)
    let tfilm = (conductor_temperature + ambient_temperature) / 2.0;

    // Absolute Viscosity of Air (m_f), (lb/ft*h)
    // dynamic_viscosity
    // Equation 13b
    let uf = 0.00353 * (tfilm + 273.15).powf(1.5) / (tfilm + 383.4);

    // air_density (lb/ft^3)
    // Equation 14b
    let pf = (0.080695 - 2.901e-6 * elevation + 3.7e-11 * elevation.powi(2)) / (1.0 + 0.00367 * tfilm);

    // Equation 4a Section 4.4.3.1, page 11.
    let kangle = 1.194
        - wind_angle_rad.cos()
        + 0.194 * (2.0 * wind_angle_rad).cos()
        + 0.368 * (2.0 * wind_angle_rad).sin();

    // Equation 2c
    let nre = diameter
        * pf
        * (wind_speed * 60.0 * 60.0) // Because dynamic_viscosity is in lb/ft-hr, we must convert wind speed to ft/hr.
        / uf;

    // thermal_conductivity_of_air
    // Equation 15b
    let kf = 7.388e-3 + 2.279e-5 * tfilm - 1.343e-9 * tfilm.powi(2);

    // Section 4.4.3.2, eq 5a 5b, page 12
    // qc0 = natural_convection
    let qc0 = 1.825
        * pf.powf(0.5)
        * diameter.powf(0.75)
        * (conductor_temperature - ambient_temperature).powf(1.25);

    // Equation 3a
    let qc1 = kangle
        * (1.01 + 1.35 * nre.powf(0.52))
        * kf
        * (conductor_temperature - ambient_temperature);

    // Equation 3b
    let qc2 = kangle
        * 0.754 * nre.powf(0.6)
        * kf
        * (conductor_temperature - ambient_temperature);

    // IEEE 738 recommends taking max of 3a / 3b results.
    // The convective heat loss is the bigger of forced and natural convection
    // From section 4.4.3 in the standard, page 10.
    f64::max(qc0, f64::max(qc1, qc2))
}

/// Returns radiated_heat_loss Qr (Watts / ft)
/// # Arguments
/// * `ambient_temperature` - T_a: Degrees (C)
/// * `conductor_temperature` - T_s: Conductor Surface Temperature (C)
/// * `emissivity` - ε: Epsilon, Emissivity of conductor (0.0 to 1.0)
/// * `diameter` - D_0: Outer diameter of the conductor (ft)
pub fn radiated_heat_loss(
    ambient_temperature: f64,
    conductor_temperature: f64,
    emissivity: f64,
    diameter: f64,
) -> f64 {
    // Section 4.4.4, eq 7a 7b, page 12
    1.656
        * diameter
        * emissivity
        * (
            ((conductor_temperature + 273.0) / 100.0).powi(4)
            - ((ambient_temperature + 273.0) / 100.0).powi(4)
        )
}

/// Calculates day of year. 
/// # Arguments
/// * `month` - Month January (1) to December (12)
/// * `day_of_month` - Day of Month, 1 to 31
pub fn day_of_year(month: i32, day_of_month: i32) -> i32 {
    let days_in_month = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    
    let mut result = day_of_month;
    for i in 1..month {
        result += days_in_month[i as usize];
    }
    result
}

/// Returns solar_heat_gain Qs (Watts / ft)
/// # Arguments
/// * `solar_radiation` - w/ft^2, or <0 if it should be calculated via month/day/hour
/// * `month` - 1 (January) to 12 (December)
/// * `day_of_month` - Day of Month (1-31)
/// * `hour_of_day` - Hour of Day, 0 to 23 (e.g. 11:00 AM => 11)
/// * `latitude_deg` - Lat: Latitude (Decimal Degrees)
/// * `line_azimuth_deg` - Z_l: If line runs E-W => 90 Degrees
/// * `elevation` - H_e: Height of conductor above sea level (ft)
/// * `atmosphere_clear` - Clear? (True) Industrial? (False)
/// * `absorptivity` - α: Alpha, Absorptivity of conductor (0.0 to 1.0)
/// * `diameter` - D_0: Outer diameter of the conductor (ft)
pub fn solar_heat_gain(
    solar_radiation: f64,
    month: i32,
    day_of_month: i32,
    hour_of_day: f64,
    latitude_deg: f64, 
    line_azimuth_deg: f64,
    elevation: f64, 
    atmosphere_clear: bool,
    absorptivity: f64,
    diameter: f64, 
) -> f64 {
    // If solar radiation is already specified, immediately return the value.
    if solar_radiation >= 0.0 {
        return absorptivity * solar_radiation * diameter;
    }

    // Constants
    let pi = std::f64::consts::PI;

    let day_of_year = day_of_year(month, day_of_month);

    let latitude_rad = latitude_deg * (pi / 180.0);

    // Hour angle relative to noon, 15*(Time-12), at 11AM, Time = 11 and the Hour angle= –15 deg 
    let w_deg = (hour_of_day - 12.0) * 15.0;
    let w_rad = w_deg * (pi / 180.0);

    // Table 3 - Atmosphere condition coefficients
    let (a, b, c, d, e, f, g) = match atmosphere_clear {
        true => (-3.9241, 5.9276, -1.7856e-1, 3.223e-3, -3.3549e-5, 1.8053e-7, -3.7868e-10),
        false => (4.9408, 1.3208, 6.1444e-2, -2.9411e-3, 5.07752e-5, -4.03627e-7, 1.22967e-9),
    };

    // Table H.5 - Solar heat multiplying factors, Ksolar for high altitudes
    let mult = match elevation {
        _ if elevation > 15000.0 => 1.3,
        _ if elevation > 10000.0 => 1.25,
        _ if elevation > 5000.0 => 1.15,
        _ => 1.0,
    };

    // Equation 16b - 23.4583 more precisely from Annex A
    let p_rad = (((284.0 + (day_of_year as f64)) / 365.0) * 360.0) * (pi / 180.0);
    let delta_rad = (23.4583 * p_rad.sin()) * (pi / 180.0);

    // Equation 16a
    let hc_rad = (latitude_rad.cos() * delta_rad.cos() * w_rad.cos() + latitude_rad.sin() * delta_rad.sin()).asin();
    // Limit to 0-90 range. Convert to degrees.
    let hc_deg = hc_rad * (180.0 / pi);

    // Equation 18 - Total solar and sky radiated heat intensity
    let qs = a + b * hc_deg + c * hc_deg.powi(2) + d * hc_deg.powi(3) + e * hc_deg.powi(4) + f * hc_deg.powi(5) + g * hc_deg.powi(6);

    // Equation 20 - Solar altitude correction factor
    let ksolar = 1.0 + 3.5e-5 * elevation - 1.0e-9 * elevation.powi(2);

    // Equation 8 - Total solar and sky radiated heat intensity corrected for elevation
    // Qs sometimes can compute as less than 0, if the sun is down. The lowest heating you can have is 0.
    let qse = f64::max(qs,0.0) * mult * ksolar;

    // Equation 17b
    let x = w_rad.sin() / ((latitude_rad.sin() * w_rad.cos() - latitude_rad.cos() * delta_rad.tan()));

    let cc_deg = 
        if -180.0 <= w_deg && w_deg < 0.0 {
            if x >= 0.0 { 0.0 } 
            else { 180.0 }
        } else {
            if x < 0.0 { 180.0 } 
            else { 360.0 }
        };

    let cc_rad = cc_deg * (pi / 180.0);

    // Azimuth of line
    let zl_rad = line_azimuth_deg * (pi / 180.0);

    // Azimuth of sun
    let zc_rad = cc_rad + (x).atan();

    // Equation 9 - Effective angle of incidence of the sun’s rays
    let theta = (hc_rad.cos() * (zc_rad - zl_rad).cos()).acos();

    // Compute solar_heat_flux
    absorptivity * qse * (theta).sin() * diameter
}

/// Returns resistance, adjusted to given conductor_temperature.
/// # Arguments
/// * `conductor_temperature` - T_s: Conductor Surface Temperature (C)
/// * `t_low` - Low Temperature, Degrees C
/// * `t_high` - High Temperature, Degrees C
/// * `r_low` - Resistance at Low Temperature, Ohms
/// * `r_high` - Resistance at High Temperature, Ohms
pub fn adjust_r(
    conductor_temperature: f64, 
    t_low: f64, 
    t_high: f64, 
    r_low: f64, 
    r_high: f64, 
) -> f64 {
    // Equation 10
    let ohms_per_c: f64 = (r_high - r_low) / (t_high - t_low);
    (ohms_per_c * (conductor_temperature - t_low)) + r_low
}

/// Returns thermal_rating (Amps)
/// # Arguments
/// * `solar_radiation` - w/ft^2, or <0 if it should be calculated via month/day/hour
/// * `month` - 1 (January) to 12 (December)
/// * `day_of_month` - Day of Month (1-31)
/// * `hour_of_day` - Hour of Day, 0 to 23 (e.g. 11:00 AM => 11)
/// * `ambient_temperature` - T_a: Degrees (C)
/// * `wind_speed` - V_w: Wind Speed (ft/s)
/// * `wind_angle_deg` - Wind Angle (Degrees) 0 to 90
/// * `latitude_deg` - Lat: Latitude (Decimal Degrees)
/// * `line_azimuth_deg` - Z_l: If line runs E-W => 90 Degrees
/// * `elevation` - H_e: Height of conductor above sea level (ft)
/// * `atmosphere_clear` - Clear? (True) Industrial? (False)
/// * `conductor_temperature` - T_s: Conductor Surface Temperature (C)
/// * `absorptivity` - α: Alpha, Absorptivity of conductor (0.0 to 1.0)
/// * `emissivity` - ε: Epsilon, Emissivity of conductor (0.0 to 1.0)
/// * `diameter` - D_0: Outer diameter of the conductor (ft)
/// * `t_low` - Low Temperature, Degrees C
/// * `t_high` - High Temperature, Degrees C
/// * `r_low` - Resistance at Low Temperature, Ohms
/// * `r_high` - Resistance at High Temperature, Ohms
pub fn thermal_rating(
    solar_radiation: f64,
    month: i32,
    day_of_month: i32,
    hour_of_day: f64,
    ambient_temperature: f64,
    wind_speed: f64,
    wind_angle_deg: f64,
    latitude_deg: f64,
    line_azimuth_deg: f64,
    elevation: f64,
    atmosphere_clear: bool,
    conductor_temperature: f64,
    absorptivity: f64,
    emissivity: f64,
    diameter: f64,
    t_low: f64,
    t_high: f64,
    r_low: f64,
    r_high: f64,
) -> f64 {

    if conductor_temperature < ambient_temperature {
        return 0.0;
    }

    let qc = convective_heat_loss(ambient_temperature,wind_speed,wind_angle_deg,elevation,conductor_temperature,diameter);

    let qr = radiated_heat_loss(ambient_temperature,conductor_temperature,emissivity,diameter);

    let qs: f64 = solar_heat_gain(solar_radiation,month,day_of_month,hour_of_day,latitude_deg,line_azimuth_deg,elevation,atmosphere_clear,absorptivity,diameter);

    let r = adjust_r(conductor_temperature,t_low,t_high,r_low,r_high);

    if qc + qr - qs < 0 {
        // The ambient temperature + solar heating, has brought the conductor to a higher temperature than the specified MOT "conductor_temperature"
        return 0.0;
    }

    ((qc + qr - qs) / r).powf(0.5)
}



/// Returns calculated_temperature (C) based on input conditions
/// # Arguments
/// * `solar_radiation` - w/ft^2, or <0 if it should be calculated via month/day/hour
/// * `month` - 1 (January) to 12 (December)
/// * `day_of_month` - Day of Month (1-31)
/// * `hour_of_day` - Hour of Day, 0 to 23 (e.g. 11:00 AM => 11)
/// * `ambient_temperature` - T_a: Degrees (C)
/// * `wind_speed` - V_w: Wind Speed (ft/s)
/// * `wind_angle_deg` - Wind Angle (Degrees) 0 to 90
/// * `latitude_deg` - Lat: Latitude (Decimal Degrees)
/// * `line_azimuth_deg` - Z_l: If line runs E-W => 90 Degrees
/// * `elevation` - H_e: Height of conductor above sea level (ft)
/// * `atmosphere_clear` - Clear? (True) Industrial? (False)
/// * `current` - Current (amps)
/// * `tolerance` - Tolerance on result (amps)
/// * `absorptivity` - α: Alpha, Absorptivity of conductor (0.0 to 1.0)
/// * `emissivity` - ε: Epsilon, Emissivity of conductor (0.0 to 1.0)
/// * `diameter` - D_0: Outer diameter of the conductor (ft)
/// * `t_low` - Low Temperature, Degrees C
/// * `t_high` - High Temperature, Degrees C
/// * `r_low` - Resistance at Low Temperature, Ohms
/// * `r_high` - Resistance at High Temperature, Ohms
pub fn calculated_temperature(
    solar_radiation: f64,
    month: i32,
    day_of_month: i32,
    hour_of_day: f64,
    ambient_temperature: f64,
    wind_speed: f64,
    wind_angle_deg: f64,
    latitude_deg: f64,
    line_azimuth_deg: f64,
    elevation: f64,
    atmosphere_clear: bool,
    current: f64,
    tolerance: f64,
    absorptivity: f64,
    emissivity: f64,
    diameter: f64,
    t_low: f64,
    t_high: f64,
    r_low: f64,
    r_high: f64,
) -> f64 {
    if current < 0.0 {
        return 0.0;
    }

    let mut lower_bound: f64 = ambient_temperature;
    let mut upper_bound: f64 = 256.0;
    let target_y: f64 = current;

    // Increase upper_bound until y(upper_bound) exceeds target_y or it becomes very large
    while thermal_rating(
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
        ,upper_bound // Modified variable
        ,absorptivity
        ,emissivity
        ,diameter
        ,t_low
        ,t_high
        ,r_low
        ,r_high
    ) < target_y && upper_bound < f64::MAX / 2.0 {
        upper_bound *= 2.0;
    }

    // Bisection search with known upper_bound and lower_bound
    while upper_bound - lower_bound > tolerance {
        let mid = (lower_bound + upper_bound) / 2.0;
        let mid_y = thermal_rating(
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
            ,mid // Modified variable
            ,absorptivity
            ,emissivity
            ,diameter
            ,t_low
            ,t_high
            ,r_low
            ,r_high
        );

        if mid_y < target_y {
            lower_bound = mid;
        } else {
            upper_bound = mid;
        }
    }

    // Return the midpoint of the final range
    (lower_bound + upper_bound) / 2.0
}


/// Returns conductor_temperature_rise (C)
/// # Arguments
/// * `solar_radiation` - w/ft^2, or <0 if it should be calculated via month/day/hour
/// * `month` - 1 (January) to 12 (December)
/// * `day_of_month` - Day of Month (1-31)
/// * `hour_of_day` - Hour of Day, 0 to 23 (e.g. 11:00 AM => 11)
/// * `ambient_temperature` - T_a: Degrees (C)
/// * `wind_speed` - V_w: Wind Speed (ft/s)
/// * `wind_angle_deg` - Wind Angle (Degrees) 0 to 90
/// * `latitude_deg` - Lat: Latitude (Decimal Degrees)
/// * `line_azimuth_deg` - Z_l: If line runs E-W => 90 Degrees
/// * `elevation` - H_e: Height of conductor above sea level (ft)
/// * `atmosphere_clear` - Clear? (True) Industrial? (False)
/// * `conductor_temperature` - Initial Conductor Surface Temperature (C)
/// * `current` - Current (amps)
/// * `time_step` - Timestep (seconds)
/// * `steps` - Number of time steps to apply
/// * `absorptivity` - α: Alpha, Absorptivity of conductor (0.0 to 1.0)
/// * `emissivity` - ε: Epsilon, Emissivity of conductor (0.0 to 1.0)
/// * `diameter` - D_0: Outer diameter of the conductor (ft)
/// * `t_low` - Low Temperature, Degrees C
/// * `t_high` - High Temperature, Degrees C
/// * `r_low` - Resistance at Low Temperature, Ohms
/// * `r_high` - Resistance at High Temperature, Ohms
/// * `heat_capacity` - m*Cp: Total heat capacity of conductor (J/(ft-°C))
pub fn conductor_temperature_rise(
    solar_radiation: f64,
    month: i32,
    day_of_month: i32,
    hour_of_day: f64,
    ambient_temperature: f64,
    wind_speed: f64,
    wind_angle_deg: f64,
    latitude_deg: f64,
    line_azimuth_deg: f64,
    elevation: f64,
    atmosphere_clear: bool,
    conductor_temperature: f64,
    current: f64,
    time_step: f64,
    steps: i32,
    absorptivity: f64,
    emissivity: f64,
    diameter: f64,
    t_low: f64,
    t_high: f64,
    r_low: f64,
    r_high: f64,
    heat_capacity: f64,
) -> f64 {

    if conductor_temperature < ambient_temperature {
        return 0.0;
    }

    let mut final_temperature = conductor_temperature;

    for _ in 0..steps {
        let qc = convective_heat_loss(ambient_temperature,wind_speed,wind_angle_deg,elevation,final_temperature,diameter);
        let qr = radiated_heat_loss(ambient_temperature,final_temperature,emissivity,diameter);
        let qs: f64 = solar_heat_gain(solar_radiation,month,day_of_month,hour_of_day,latitude_deg,line_azimuth_deg,elevation,atmosphere_clear,absorptivity,diameter);
        let r = adjust_r(final_temperature,t_low,t_high,r_low,r_high);
        let delta_t = ((r * current.powf(2.0)) + qs - qc - qr) * time_step / heat_capacity;
        final_temperature += delta_t;
    }

    final_temperature - conductor_temperature
}

/// Returns transient_rating (Amps)
/// # Arguments
/// * `solar_radiation` - w/ft^2, or <0 if it should be calculated via month/day/hour
/// * `month` - 1 (January) to 12 (December)
/// * `day_of_month` - Day of Month (1-31)
/// * `hour_of_day` - Hour of Day, 0 to 23 (e.g. 11:00 AM => 11)
/// * `ambient_temperature` - T_a: Degrees (C)
/// * `wind_speed` - V_w: Wind Speed (ft/s)
/// * `wind_angle_deg` - Wind Angle (Degrees) 0 to 90
/// * `latitude_deg` - Lat: Latitude (Decimal Degrees)
/// * `line_azimuth_deg` - Z_l: If line runs E-W => 90 Degrees
/// * `elevation` - H_e: Height of conductor above sea level (ft)
/// * `atmosphere_clear` - Clear? (True) Industrial? (False)
/// * `conductor_temperature` - Initial Conductor Surface Temperature (C)
/// * `conductor_temperature_max` - Max Final Conductor Surface Temperature (C)
/// * `time_step` - Timestep (seconds)
/// * `steps` - Number of time steps to apply
/// * `tolerance` - Tolerance on result (amps)
/// * `absorptivity` - α: Alpha, Absorptivity of conductor (0.0 to 1.0)
/// * `emissivity` - ε: Epsilon, Emissivity of conductor (0.0 to 1.0)
/// * `diameter` - D_0: Outer diameter of the conductor (ft)
/// * `t_low` - Low Temperature, Degrees C
/// * `t_high` - High Temperature, Degrees C
/// * `r_low` - Resistance at Low Temperature, Ohms
/// * `r_high` - Resistance at High Temperature, Ohms
/// * `heat_capacity` - m*Cp: Total heat capacity of conductor (J/(ft-°C))
pub fn transient_rating(
    solar_radiation: f64,
    month: i32,
    day_of_month: i32,
    hour_of_day: f64,
    ambient_temperature: f64,
    wind_speed: f64,
    wind_angle_deg: f64,
    latitude_deg: f64,
    line_azimuth_deg: f64,
    elevation: f64,
    atmosphere_clear: bool,
    conductor_temperature: f64,
    conductor_temperature_max: f64,
    time_step: f64,
    steps: i32,
    tolerance: f64,
    absorptivity: f64,
    emissivity: f64,
    diameter: f64,
    t_low: f64,
    t_high: f64,
    r_low: f64,
    r_high: f64,
    heat_capacity: f64,
) -> f64 {

    if conductor_temperature_max < conductor_temperature {
        return 0.0;
    }

    // Assume the rating is somewhere between 0 to 4096A.
    let mut lower_bound: f64 = 0.0;
    let mut upper_bound: f64 = 4096.0;
    // delta_t_max
    let target_y: f64 = conductor_temperature_max - conductor_temperature; 

    // Increase upper_bound until y(upper_bound) exceeds target_y or it becomes very large
    while conductor_temperature_rise(
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
            ,upper_bound // Modified variable
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
        ) < target_y && upper_bound < f64::MAX / 2.0 {
        upper_bound *= 2.0;
    }

    // Bisection search with known upper_bound and lower_bound
    while upper_bound - lower_bound > tolerance {
        let mid = (lower_bound + upper_bound) / 2.0;
        let mid_y = conductor_temperature_rise(
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
            ,mid // Modified variable
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

        if mid_y < target_y {
            lower_bound = mid;
        } else {
            upper_bound = mid;
        }
    }

    // Return the midpoint of the final range
    (lower_bound + upper_bound) / 2.0
}

