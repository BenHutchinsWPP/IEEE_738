Function convective_heat_loss( _
    ByVal ambient_temperature As Double, _
    ByVal wind_speed As Double, _
    ByVal wind_angle_deg As Double, _
    ByVal elevation As Double, _
    ByVal conductor_temperature As Double, _
    ByVal diameter As Double _
) As Double
    Dim pi As Double
    Dim wind_angle_deg_limited As Double
    Dim wind_angle_rad As Double
    Dim tfilm As Double
    Dim uf As Double
    Dim pf As Double
    Dim kangle As Double
    Dim nre As Double
    Dim kf As Double
    Dim qc0 As Double
    Dim qc1 As Double
    Dim qc2 As Double

    pi = Application.WorksheetFunction.pi()
    wind_angle_deg_limited = 90 - Abs((wind_angle_deg Mod 180) - 90)
    wind_angle_rad = wind_angle_deg * (pi / 180)
    tfilm = (conductor_temperature + ambient_temperature) / 2
    uf = 0.00353 * (tfilm + 273.15) ^ 1.5 / (tfilm + 383.4)
    pf = (0.080695 - 0.000002901 * elevation + 0.000000000037 * elevation ^ 2) / (1 + 0.00367 * tfilm)
    kangle = 1.194 - Cos(wind_angle_rad) + 0.194 * Cos(2 * wind_angle_rad) + 0.368 * Sin(2 * wind_angle_rad)
    nre = diameter * pf * (wind_speed * 60 * 60) / uf ' Convert wind speed to ft/hr
    kf = 0.007388 + 0.00002279 * tfilm - 0.000000001343 * tfilm ^ 2
    qc0 = 1.825 * Sqr(pf) * diameter ^ 0.75 * (conductor_temperature - ambient_temperature) ^ 1.25
    qc1 = kangle * (1.01 + 1.35 * nre ^ 0.52) * kf * (conductor_temperature - ambient_temperature)
    qc2 = kangle * 0.754 * nre ^ 0.6 * kf * (conductor_temperature - ambient_temperature)

    ' Return the maximum value between qc0, qc1, and qc2
    convective_heat_loss = Application.WorksheetFunction.Max(qc0, qc1, qc2)
End Function

Function radiated_heat_loss( _
    ByVal ambient_temperature As Double, _
    ByVal conductor_temperature As Double, _
    ByVal emissivity As Double, _
    ByVal diameter As Double _
) As Double
    radiated_heat_loss = 1.656 _
        * diameter _
        * emissivity _
        * ( _
            ((conductor_temperature + 273) / 100) ^ 4 _
            - ((ambient_temperature + 273) / 100) ^ 4 _
          )
End Function

Function day_of_year(ByVal month As Integer, ByVal day_of_month As Integer) As Integer
    Dim days_in_month() As Variant
    Dim result As Integer
    Dim i As Integer

    ' Define days in each month
    days_in_month = Array(0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31)

    ' Initialize result with day of the month
    result = day_of_month

    ' Sum days for each month up to specified month
    For i = 1 To month - 1
        result = result + days_in_month(i)
    Next i

    day_of_year = result
End Function

Function solar_heat_gain( _
    ByVal month As Integer, _
    ByVal day_of_month As Integer, _
    ByVal hour_of_day As Double, _
    ByVal latitude_deg As Double, _
    ByVal line_azimuth_deg As Double, _
    ByVal elevation As Double, _
    ByVal atmosphere_clear As Boolean, _
    ByVal absorptivity As Double, _
    ByVal diameter As Double _
) As Double
    Dim pi As Double
    Dim day_of_yr As Integer
    Dim latitude_rad As Double
    Dim w_deg As Double
    Dim w_rad As Double
    Dim a As Double
    Dim b As Double
    Dim c As Double
    Dim d As Double
    Dim e As Double
    Dim f As Double
    Dim g As Double
    Dim mult As Double
    Dim p_rad As Double
    Dim delta_rad As Double
    Dim hc_rad As Double
    Dim hc_deg As Double
    Dim qs As Double
    Dim ksolar As Double
    Dim qse As Double
    Dim x As Double
    Dim cc_deg As Double
    Dim cc_rad As Double
    Dim zl_rad As Double
    Dim zc_rad As Double
    Dim theta As Double

    pi = Application.WorksheetFunction.pi()
    day_of_yr = day_of_year(month, day_of_month)
    latitude_rad = latitude_deg * (pi / 180)
    w_deg = (hour_of_day - 12) * 15
    w_rad = w_deg * (pi / 180)

    ' Atmosphere condition coefficients
    If atmosphere_clear Then
        a = -3.9241: b = 5.9276: c = -0.17856: d = 0.003223: e = -0.000033549: f = 0.00000018053: g = -0.00000000037868
    Else
        a = 4.9408: b = 1.3208: c = 0.061444: d = -0.0029411: e = 0.0000507752: f = -0.000000403627: g = 0.00000000122967
    End If

    ' Solar heat multiplying factors for high altitudes
    If elevation > 15000 Then
        mult = 1.3
    ElseIf elevation > 10000 Then
        mult = 1.25
    ElseIf elevation > 5000 Then
        mult = 1.15
    Else
        mult = 1#
    End If

    p_rad = (((284 + day_of_yr) / 365) * 360) * (pi / 180)
    delta_rad = (23.4583 * Sin(p_rad)) * (pi / 180)
    hc_rad = Application.WorksheetFunction.Asin(Cos(latitude_rad) * Cos(delta_rad) * Cos(w_rad) + Sin(latitude_rad) * Sin(delta_rad))
    hc_deg = hc_rad * (180 / pi)
    qs = a + b * hc_deg + c * hc_deg ^ 2 + d * hc_deg ^ 3 + e * hc_deg ^ 4 + f * hc_deg ^ 5 + g * hc_deg ^ 6
    ksolar = 1 + 0.000035 * elevation - 0.000000001 * elevation ^ 2
    qse = Application.WorksheetFunction.Max(qs, 0) * mult * ksolar

    x = Sin(w_rad) / (Sin(latitude_rad) * Cos(w_rad) - Cos(latitude_rad) * Tan(delta_rad))
    If -180 <= w_deg And w_deg < 0 Then
        If x >= 0 Then
            cc_deg = 0
        Else
            cc_deg = 180
        End If
    Else
        If x < 0 Then
            cc_deg = 180
        Else
            cc_deg = 360
        End If
    End If

    cc_rad = cc_deg * (pi / 180)
    zl_rad = line_azimuth_deg * (pi / 180)
    zc_rad = cc_rad + Atn(x)
    theta = Application.WorksheetFunction.Acos(Cos(hc_rad) * Cos(zc_rad - zl_rad))

    solar_heat_gain = absorptivity * qse * Sin(theta) * diameter
End Function

Function adjust_r( _
    ByVal conductor_temperature As Double, _
    ByVal t_low As Double, _
    ByVal t_high As Double, _
    ByVal r_low As Double, _
    ByVal r_high As Double _
) As Double
    ' Calculate resistance adjustment based on Equation 10
    Dim ohms_per_c As Double
    ohms_per_c = (r_high - r_low) / (t_high - t_low)
    adjust_r = (ohms_per_c * (conductor_temperature - t_low)) + r_low
End Function

Function thermal_rating( _
    ByVal month As Integer, _
    ByVal day_of_month As Integer, _
    ByVal hour_of_day As Double, _
    ByVal ambient_temperature As Double, _
    ByVal wind_speed As Double, _
    ByVal wind_angle_deg As Double, _
    ByVal latitude_deg As Double, _
    ByVal line_azimuth_deg As Double, _
    ByVal elevation As Double, _
    ByVal atmosphere_clear As Boolean, _
    ByVal conductor_temperature As Double, _
    ByVal absorptivity As Double, _
    ByVal emissivity As Double, _
    ByVal diameter As Double, _
    ByVal t_low As Double, _
    ByVal t_high As Double, _
    ByVal r_low As Double, _
    ByVal r_high As Double _
) As Double
    Dim qc As Double, qr As Double, qs As Double, r As Double
    
    ' Calculate convective heat loss
    qc = convective_heat_loss(ambient_temperature, wind_speed, wind_angle_deg, elevation, conductor_temperature, diameter)
    
    ' Calculate radiated heat loss
    qr = radiated_heat_loss(ambient_temperature, conductor_temperature, emissivity, diameter)
    
    ' Calculate solar heat gain
    qs = solar_heat_gain(month, day_of_month, hour_of_day, latitude_deg, line_azimuth_deg, elevation, atmosphere_clear, absorptivity, diameter)
    
    ' Adjust resistance
    r = adjust_r(conductor_temperature, t_low, t_high, r_low, r_high)
    
    ' Calculate and return thermal rating
    thermal_rating = Sqr((qc + qr - qs) / r)
End Function




