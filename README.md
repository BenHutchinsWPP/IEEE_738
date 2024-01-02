# IEEE 738

This repository is intended to host files which will help utilities with calculating overhead line-ratings for IEEE 738 purposes.

WARNING: No warranty is provided for this code-base. It is provided as-is, and should have engineer and code-review prior to direct usage. 

Some useful links include:
- [FERC 881 Python Demo](https://www.ferc.gov/media/demonstration-potential-datacalculation-workflows-under-ferc-order-no-881s-ambient-adjusted)
- [Tomas Barton IEEE 738 Python Code](https://github.com/tommz9/pylinerating)
- [PJM's Overhead Conductor Ratings Sheet](https://www.pjm.com/-/media/planning/design-engineering/maac-standards/oht-cond-rating-spreadsheet.ashx)
- [FERC Order 881](https://www.ferc.gov/media/e-1-rm20-16-000)
- [IEEE 738 Standard](https://standards.ieee.org/ieee/738/4997/)
- [CIGRE TB 299](https://e-cigre.org/publication/299-guide-for-the-selection-of-weather-parameters-for-bare-overhead-conductor-ratings)

# IEEE 738 Ratings Calculations

## Steady State
In steady-state, our thermal balance is dictated by the following equation:

(1a): $q_c + q_r = q_s + I^2 \cdot R$

If we solve (1a) for $I$, we get 

(1b): $I = \sqrt{\frac{q_c + q_r - q_s}{R}}$

Then, to compute:
- $I$: Current (Amps)

We will need to identify these values:
- $q_c$: `convective_heat_loss()`, Convection heat loss rate per unit length ($\frac{W}{ft}$)
- $q_r$: `radiated_heat_loss()`, Radiated heat loss rate per unit length ($\frac{W}{ft}$)
- $q_s$: `solar_heat_gain()`, Heat gain rate from sun ($\frac{W}{ft}$)
- $R$: `adjust_r()` Resistance (Ohms/ft) at the conductor's present temperature

There are two main functions available in this library for computing things in steady-state:
- `thermal_rating()`: Calculates Current (Amps) given Conductor Temperature
- `calculated_temperature()`: Calculates Conductor Temperature given Current (Amps)

### thermal_rating()
Returns the steady-state current in Amps which will cause the conductor to reach the given conductor_temperature under the given weather conditions, using the following inputs:

|Symbol|Type|Variable|Description|
|----|----|----|----|
|$Q_{se}$|f64|`solar_radiation`|$w/ft^2$ or <0 if it should be derived from month/day/hour
||i32|`month`|1 (January) to 12 (December)
||i32|`day_of_month`|Day of Month (1-31)
||f64|`hour_of_day`|Hour of Day, 0 to 23 (e.g. 11:00 AM => 11)
|$T_a$|f64|`ambient_temperature`|Degrees (C)
|$V_w$|f64|`wind_speed`|Wind Speed (ft/s)
||f64|`wind_angle_deg`|Wind Angle (Degrees) 0 to 90
|$Lat$|f64|`latitude_deg`|Latitude (Decimal Degrees)
|$Z_l$|f64|`line_azimuth_deg`|If line runs E-W => 90 Degrees
|$H_e$|f64|`elevation`|Height of conductor above sea level (ft)
||bool|`atmosphere_clear`|Clear? (True) Industrial? (False)
|$T_s$|f64|`conductor_temperature`|Conductor Surface Temperature (C)
|$\alpha$|f64|`absorptivity`|Alpha, Absorptivity of conductor (0.0 to 1.0)
|$\epsilon$|f64|`emissivity`|Epsilon, Emissivity of conductor (0.0 to 1.0)
|$D_0$|f64|`diameter`|Outer diameter of the conductor (ft)
|$T_{low}$|f64|`t_low`|Low Temperature (C)
|$T_{high}$|f64|`t_high`|High Temperature (C)
|$R_{low}$|f64|`r_low`|Resistance at Low Temperature, ($\frac{Ohms}{ft}$)
|$R_{high}$|f64|`r_high`|Resistance at High Temperature, ($\frac{Ohms}{ft}$)

#### Example Hand-Calculation Below for Peer-Check Purposes
- Drake 795 ACSR Conductor
- June 10th @ 11 AM
- Clear Sky
- Alpha = 0.8, Epsilon = 0.8
- Maximum Operating Temperature = 100C (MOT)
- Ambient Temperature = 40C
- Wind = 2 ft/s
- Wind Angle = 90 Degrees to Conductor
- Elevation = Sea Level (0 ft)


|Symbol|Variable|Value|
|----|----|----|
|$Q_{se}$|solar_radiation | -1.0 (i.e. a negative value here indicates this is to be calculated based on time of day instead of overridden)|
||month | 6 |
||day_of_month | 10 |
||hour_of_day | 11.0 |
|$T_a$|ambient_temperature | 40.0 |
|$V_w$|wind_speed | 2.0 |
||wind_angle_deg: f64 | 90.0 |
|$Lat$|latitude_deg: f64 | 30.0 |
|$Z_l$|line_azimuth_deg: f64 | 90.0 |
|$H_e$|elevation: f64 | 0.0 |
||atmosphere_clear: bool | true |
|$T_s$|conductor_temperature: f64 | 100.0 |
|$\alpha$|absorptivity: f64 | 0.8 |
|$\epsilon$|emissivity: f64 | 0.8 |
|$D_0$|diameter: f64 | 0.092333333 |
|$T_{low}$|t_low: f64 | 25.0 |
|$T_{high}$|t_high: f64 | 75.0 |
|$R_{low}$|r_low: f64 | 2.20833e-05 |
|$R_{high}$|r_high: f64 | 2.63258e-05 |

To compute current (based on 1a) we need to first compute:
- $q_c$: `convective_heat_loss()`
- $q_r$: `radiated_heat_loss()`
- $q_s$: `solar_heat_gain()`
- $R$: `adjust_r()` Resistance (Ohms/ft)

$q_c$: `convective_heat_loss()`
- Limit `wind_angle_deg` ($\phi$) to between 0-90 degrees.
- $\phi = 90-abs(mod(\phi,180) - 90)$
- $T_{film} = \frac{T_s+T_a}{2}$  (6)
- $T_{film}$ = 70.0 (degrees C)
- $\mu_f=\frac{0.00353 \cdot (T_{film} + 273.15)^{1.5}}{(T_{film} + 383.4)}$ (13b Dynamic viscosity of air)
- $\mu_f$ = 0.049490198353345498 (lb/ft - hour) 
- $\rho_f=\frac{0.080695 - 2.901 \cdot 10^{-6} \cdot H_e + 3.7 \cdot 10^{-11} \cdot H_e^2}{1 + 0.00367 \cdot T_{film}}$ (14b Air density)
- $\rho_f$ = 0.064201607128649862 (lb/ft^3)
- $K_{angle} = 1.194 - \cos(\phi) + 0.194 \cdot \cos(2\phi) + 0.368 \cdot \sin(2\phi)$ (4a Section 4.4.3.1, page 11)
- $K_{angle}$ = 1.0
- $N_{Re} = \frac{D_0 \cdot \rho_f \cdot V_w}{\mu_f}$ (2c Reynolds Number)
  - Note: Because Dynamic viscosity is in lb/ft-hr, we must convert wind speed to ft/hr.
- $N_{Re}$ = 862.41780564933526
- $k_f = 7.388 \cdot 10^{-3} + 2.279 \cdot 10^{-5} \cdot T_{\text{film}} - 1.343 \cdot 10^{-9} \cdot T_{\text{film}}^2$ (W / ft * Degrees C) (15b Thermal conductivity of air)
- $k_f$ = 0.0089767192999999998
- $q_{c0} = 1.825 \cdot \rho_f^{0.5} \cdot D_0^{0.75} \cdot (T_s - T_a)^{1.25}$  (W/ft) (Section 4.4.3.2, 5b, page 12, Natural Convection)
- $q_{c0}$ = 12.934324909542022
- $q_{c1} = K_{\text{angle}} \left[ 1.01 + 1.35 \cdot N_{\text{Re}}^{0.52} \right] \cdot k_f \cdot (T_s - T_a)$ (3a Forced convection - correct at low winds)
- $q_{c1}$ = 24.988191839976331 (W/ft)
- $q_{c2} = K_{angle} \cdot 0.0754 \cdot N_{Re}^{0.6} \cdot k_f \cdot (T_s - T_a)$ (3b Forced convection - correct at high winds)
- $q_{c2}$ = 23.446113878522919 (W/ft)
- $q_c = Max(q_{c0},q_{c1},q_{c2})$ = 24.988191839976331 (W/ft)

$q_r$: `radiated_heat_loss()`
- $q_r = 1.656 \cdot D_0 \cdot \varepsilon \cdot [(\frac{T_s + 273}{100})^4 - (\frac{T_a + 273}{100})^4]$ (Section 4.4.4, eq 7a 7b, page 12)
  - Note: PJM's ratings calculations use 273.15 here instead of 273.
- $q_r$ = 11.937464384798224 (W/ft)

$q_s$: `solar_heat_gain()`
- If solar radiation ($Q_{se}$) is already specified, immediately return $q_s = \alpha \cdot Q_{se} \cdot D_0$.
- $N = (31 + 28 + 31 + 30 + 31) + 10$ (Day of Year)
- $N$ = 161
- $\omega = (Time - 12.0) * 15.0$ (Hour angle relative to noon, e.g. at 11AM, Time = 11 and the Hour angle= –15 deg)
- $\omega$ = -15 (Degrees)
- Table 3 - Atmosphere condition coefficients
  - Selected column for "Clear" skies

|Variable|Clear|Industrial|
|-|-|-|
|a|-3.9241|4.9408|
|b|5.9276|1.3208|
|c|-1.79e-1|6.14e-2|
|d|3.22e-3|-2.94e-3|
|e|-3.35e-5|5.08e-5|
|f|1.81e-7|-4.04e-7|
|g|-3.79e-10|1.23e-9|

- Table H.5 - Solar heat multiplying factors, for high altitudes
  - $mult = 1.0$

|Elevation|$K_{solar}$|
|-|-|
|$H_e > 15,000 ft$|1.30|
|$H_e > 10,000 ft$|1.25|
|$H_e > 5,000 ft$|1.15|
|$H_e > 0 ft$|1.00|

- $\delta = 23.46 \cdot \sin \left[ \frac{284 + N}{365} \cdot 360 \right]$ (16b - 23.4583 was taken from Annex A for higher precision)
- $\delta$ = 0.40177098151385465 (rad)
- $H_c = \arcsin \left[ \cos(\text{Lat}) \cdot \cos(\delta) \cdot \cos(\omega) + \sin(\text{Lat}) \cdot \sin(\delta) \right]
$ (16a - Altitude of the sun)
- $H_c$ = 74.890380558702674 (deg)
- $Q_s = A + B H_c + C H_c^2 + D H_c^3 + E H_c^4 + F H_c^5 + G H_c^6$ (18 - Total solar and sky radiated heat intensity)
- $Q_s$ = 95.43742328317225 (w/ft^2)
- $K_{solar}=A+B \cdot H_e + C \cdot H_e^2$ (20 - Elevation correction factor)
  - (A=1, B=3.5e-5, C=1.0e-9)
- $K_{solar}$ = 1.0
- $Q_{se}=max(q_s,0) * mult * k_{solar}$ (8 - Total solar and sky radiated heat intensity corrected for elevation)
  - Note: $q_s$ can come out negative when the sun is below the horizon. But the lowest solar heating you can have is 0. Therefore we take the max of $q_s$ and 0 here.
- $Q_{se}$ = 95.43742328317225 (w/ft^2)
- $\chi = \frac{\sin(\omega)}{\sin(\text{Lat}) \cdot \cos(\omega) - \cos(\text{Lat}) \cdot \tan(\delta)}$ (17b)
- $\chi$ = -2.2505218045476418 (rad)
- Table 2 - Solar azimuth constant, C, as a function of “Hour angle,” ω, and Solar Azimuth variable,χ 
    - $C$ = 180

|“Hour angle”, ω, degrees|C if χ ≥ 0 degrees|C if χ < 0 degrees|
|-|-|-|
|–180 ≤ ω < 0|0|180|
|0 ≤ ω < 180|180|360|

- $Z_c=C+\arctan(\chi)$ (17a - Azimuth of the sun)
- $Z_c$ = 1.9889346021863228 (rad)
- $\theta = \arccos [\cos(H_c) \cdot \cos(Z_c - Z_l)]$ (9 - Effective angle of incidence of the sun’s rays)
- $\theta$ = 1.330274712380765 (rad)
- $q_s=\alpha \cdot Q_{se} \cdot \sin(\theta) \cdot D_0$ (8 - Rate of solar heat gain)
- $q_s$ = 6.8467122146222028 (w/ft)

$R$: `adjust_r()` Resistance (Ohms/ft)
- $R = \frac{R_{high} - R_{low}} {T_{high} - T_{low}} \cdot (T - T_{low}) + R_{low}$ (10 - Conductor electrical resistance)
- $R$ = 0.000028447050000000004 (Ohms / ft)

Then, the resulting output is:

`thermal_rating()` = $\sqrt{\frac{q_c + q_r - q_s}{R}}$ = 1028.2830441942751 Amps

### calculated_temperature()
Returns the `conductor_temperature`, given an input of steady-state `current` (amps).

This is calculated via a bi-section search with a given `tolerance`, using the `thermal_rating()` routine and varying the input `conductor_temperature` until a solution is within the given tolerance.

Therefore, all the inputs are the same as the thermal_rating() function, except `conductor_temperature` has been traded out for `current` and `tolerance` as detailed below:

|Symbol|Type|Variable|Description|
|----|----|----|----|
|$I$|f64|`current`|Current (amps)|
||f64|`tolerance`|Tolerance on result (amps)|

## Transient

### conductor_temperature_rise()
Returns the temperature rise (C) of the conductor. This function takes in the same parameters as `thermal_rating()`, except for the additional parameters detailed below:

|Symbol|Type|Variable|Description|
|----|----|----|----|
|$T_{init}$|f64|`conductor_temperature`|Initial Conductor Surface Temperature (C)|
|$I_{final}$|f64|`current`|Current (amps)|
|$\Delta t$|f64|`time_step`|Timestep (seconds)|
|$n_{steps}$|f64|`steps`|Number of time steps to apply|
|$m \cdot C_p$|f64|`heat_capacity`|m*Cp: Total heat capacity of conductor (J/(ft-°C))|

This routine uses the below equation:

$q_c + q_r + m \cdot C_p \cdot \frac{dT}{dt} = q_s + I^2 \cdot R(T)$ (2b)

Where R(T) is the resistance as a function of temperature. Solving for $\Delta t$, we arrive at:

$\Delta T = \frac{R(T) \cdot I^2 + q_s - q_c - q_r}{mC_p} \cdot \Delta t$

This function relies on prior established calculations of $q_s$, $q_c$, $q_r$, and $R(T)$ to compute the equation above, then performs the number of requested time-steps to compute the final temperature of the conductor.

### transient_rating()
Returns the transient rating of the conductor ($I_{final}$) given an Amp `tolerance` and `conductor_temperature_max`.

This routine takes the same parameters as `conductor_temperature_rise()`, except for the additional parameters detailed below:

|Symbol|Type|Variable|Description|
|----|----|----|----|
|$T_{max}$|f64|`conductor_temperature_max`|Max Final Conductor Surface Temperature (C)|
||f64|`tolerance`|Tolerance on result (amps)|

It utilizes the `conductor_temperature_rise()` routine, and performs a bi-section search on the final conductor current until reaching the desired final conductor temperature. 

# Errata or To Do
- When peer-checking rating methodologies against other utility computations, it was noted that [SouthWire Rate](https://www.southwire.com/swratepro) Lite v1.0.3 may or may not be taking into account the thermal heat capacity of the wire's core for transient rating calculations; however, we were unable to replicate the issue later-on. It might be a Heisenbug, or an error on our part when verifying the calculation. Leaving this note temporarily in case the bug re-appears.

