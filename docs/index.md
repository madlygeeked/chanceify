---
title: chanceify™
hero: true
tagline: music reimagined
---

A small, fast Spotify player for your desktop. It plays your Spotify, shows your music stats, and keeps everything it saves in one folder next to the app. Nothing is sent anywhere except to Spotify and, if you turn it on, Last.fm.

You need a Spotify **Premium** account.

## Get it

1. Go to the [releases page](https://github.com/madlygeeked/chanceify/releases).
2. Download `chanceify.exe`.
3. Put it in its own folder and run it. That folder is where it keeps its settings.
4. Windows may say it "protected your PC" because the app is not signed yet. Click **More info**, then **Run anyway**.
5. Sign in with Spotify. You type your password only on Spotify's own page.

## What you can do

- **Play your Spotify.** Library, playlists, queue, search, lyrics, Connect devices.
- **Music stats.** 7 days, 30 days or all time, a streak, and a recap picture to share.
- **Last.fm scrobbling.** Plus love, and missing cover art saved for you.
- **Make it yours.** 16 themes, your own colours, colours that follow the album cover, and three visualizers: bars, flow and swirl.
- **Every key can be changed.**

{% assign anyshots = site.static_files | where_exp: "f", "f.path contains '/shots/'" %}
{% if anyshots.size > 0 %}
## Every panel

{% include gallery.html items=site.data.shots.panels %}

## Every setting

{% include gallery.html items=site.data.shots.settings %}
{% endif %}

## More

- [What it does](features)
- [Connect your own Spotify app and Last.fm](setup)
- [Settings explained](settings)
- [Keyboard shortcuts](keys)
- [Where your files live](files)
- [Questions](faq)
