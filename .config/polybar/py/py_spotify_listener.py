#!/usr/bin/env python3
# Unwrap function: (C)2011-2015 Dennis Kaarsemaker
# License: GPL3+
"""Receiver related functionality."""
import dbus.service
from os import system
from gi.repository import GLib
from dbus.mainloop.glib import DBusGMainLoop
import dbus


def catchall_handler(*args, **kwargs):
    """Catch all handler.
    Catch and print information about all singals.
    """
    print('---- Caught signal ----')
    print('%s:%s\n' % (kwargs['dbus_interface'], kwargs['member']))

    print("\n")


def quit_handler():
    """Signal handler for quitting the receiver."""
    print('Quitting....')
    loop.quit()


def event_handler(*args, **kwargs):
    data = unwrap(args)
    if 'spotify' in data[1]['Metadata']['mpris:trackid']:
        if data[1]['PlaybackStatus'] == "Playing":
            system("polybar-msg hook spotify_playpause 1")
        if data[1]['PlaybackStatus'] == "Paused":
            system("polybar-msg hook spotify_playpause 2")

        system("polybar-msg hook spotify 2")


def unwrap(val):
    if isinstance(val, dbus.ByteArray):
        return "".join([str(x) for x in val])
    if isinstance(val, (dbus.Array, list, tuple)):
        return [unwrap(x) for x in val]
    if isinstance(val, (dbus.Dictionary, dict)):
        return dict([(unwrap(x), unwrap(y)) for x, y in val.items()])
    if isinstance(val, (dbus.Signature, dbus.String)):
        return str(val)
    if isinstance(val, dbus.Boolean):
        return bool(val)
    if isinstance(val, (dbus.Int16, dbus.UInt16, dbus.Int32, dbus.UInt32, dbus.Int64, dbus.UInt64)):
        return int(val)
    if isinstance(val, dbus.Byte):
        return bytes([int(val)])
    return val


dbus.set_default_main_loop(DBusGMainLoop())
loop = GLib.MainLoop()

"""
First we get the bus to attach to. This may be either the session bus, of the
system bus. For system bus root permission is required.
We claim a bus name on the chosen bus. The name should be in form of a
domain name.
"""
bus = dbus.SessionBus()
# bus = dbus.SystemBus()
bus_name = dbus.service.BusName('sub.domain.tld', bus=bus)

"""
Attach signal handler.
Signal handlers may be attached in different ways, either by interface keyword
or DBUS interface and a signal name or member keyword.
You can easily gather all information by running the DBUS monitor.
"""
bus.add_signal_receiver(quit_handler,
                        dbus_interface='tld.domain.sub.event',
                        signal_name='quit_signal')
bus.add_signal_receiver(event_handler,
                        dbus_interface='org.freedesktop.DBus.Properties',
                        member_keyword='PropertiesChanged')

loop.run()
