import fontawesome as fa
from hereby import Here

import requests
import json

try:
    here = Here(__file__)

    with open(here.abspath('.env.json')) as key:
        API_KEY = json.load(key).get("OPENWEATHER_KEY", None)

    if API_KEY is None:
        print("No key found")
        exit(0)

    API_URL = "https://api.openweathermap.org/data/2.5/weather?q=brussels&appid={}".format(API_KEY)

    r = requests.get(API_URL)
    data = json.loads(r.content)

    weather_icons = {
        "01": "sun",
        "02": "cloud-sun",
        "03": "cloud",
        "04": "cloud",
        "O9": "cloud-rain",
        "10": "cloud-sun-rain",
        "11": "bolt",
        "13": "snowflake",
        "50": "fog",
    }

    weather_icon = data["weather"][0]['icon'][:-1]
    temperature = data["main"]['temp'] - 273.15

    print(f"{fa.icons[weather_icons[weather_icon]]} {round(temperature, 1)}°C")
except Exception as e:
    print(e)
