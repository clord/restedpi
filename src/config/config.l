%%

\b(\d\d\d\d-\d?\d-\d\dT\d?\d:\d?\d:\d?\d(\.\d+)?(Z|[-+]\d\d:\d\d))\b "date_time_z"
\b(\d\d\d\d-\d?\d-\d\dT\d?\d:\d?\d:\d?\d(\.\d+)?)\b "date_time"
\b(\d\d\d\d-\d?\d-\d\d)\b "date"

[-+]?[0-9]+(\.([0-9]+))?\b "number"
[-+]?[0-9]+\b "integer"

(::) "::"

\b(T|true)\b "true"
\b(F|false)\b "false"

\+ "+"
\- "-"
(\*|×) "*" 
[÷/] "/"
([±]|\b(plus/minus)\b) "plus/minus"
(\^|\b(xor|XOR)\b) "xor"
([&∩]|\b(and|AND)\b) "and"
(\b(not|NOT\b)|!) "not"
(\b(or|OR)\b|[∪|]) "or"
[,] ","

(\b(EQ|eq)\b|===) "eq"

(!=) "!="
(==) "=="
(<=) "<="
(>=) ">="
(<|\b(below|lt)\b) "<"
(>|\b(above|gt)\b) ">"

\[ "["
\] "]" 
\{ "{"
\} "}" 
\( "("
\) ")" 

(˚C|\bdegC)\b "degC"
(˚F|\bdegF)\b "degF"
(˚N|\bdegN)\b "degN"
(˚S|\bdegS)\b "degS"
(˚E|\bdegE)\b "degE"
(˚W|\bdegW)\b "degW"
(˚|\bdeg\b) "deg"
\bkpa\b "kpa"
\bbool\b "bool"

\bin\b "in"
\bas\b "as"
\bfn\b "fn"

\bsun_declination\b "sun_declination"
\bhour_angle_sunrise\b "hour_angle_sunrise"
\bhours_of_daylight\b "hours_of_daylight"
\bhour_of_sunrise\b "hour_of_sunrise"
\bhour_of_sunset\b "hour_of_sunset"
\bbetween\b "between"
\boffset_for_long\b "offset_for_long"
\bminute_of_hour\b  "minute_of_hour"
\bsecond_of_minute\b "second_of_minute"
\bhour_of_day\b  "hour_of_day"
\bweek_day\b "week_day"
\b(now|today)\b "now"
\bhere\b "here"
\bwhere\b "where"
\byear\b "year"
\bmonth_of_year\b "month_of_year"
\bday_of_month\b "day_of_month"
\bday_of_year\b "day_of_year"
\blerp\b "lerp"
\btrunc\b "trunc"
\bread\b "read"
(\b([a-zA-Z_])([0-9]|[a-zA-Z_])*\b|'[^']+') "identifier"

[[:space:]]+ ;

. "UNMATCHED"

