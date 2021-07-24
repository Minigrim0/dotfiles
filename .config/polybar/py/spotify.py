import dbus
from hereby import Here

trunclen = 30
here = Here(__file__)

try:
    session_bus = dbus.SessionBus()
    spotify_bus = session_bus.get_object(
        "org.mpris.MediaPlayer2.spotify",
        "/org/mpris/MediaPlayer2"
    )

    spotify_properties = dbus.Interface(
        spotify_bus, "org.freedesktop.DBus.Properties"
    )

    metadata = spotify_properties.Get(
        "org.mpris.MediaPlayer2.Player", "Metadata"
    )

    artist = str(metadata['xesam:artist'][0])
    song = str(metadata['xesam:title'])

    print(f"{artist}: {song}")

except Exception:
    print("...")
