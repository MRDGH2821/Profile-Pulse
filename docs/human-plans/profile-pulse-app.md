---
tags: []
categories: []
created: 2026-08-22T14:14:13+10:00
date: 2026-08-22
time: 14:14:13 pm
title: 2026-08-22_Profile Pulse App
---

This is a cross platform desktop & mobile app.
The primary purpose of this app is to sync profile pics from various social media platforms into the contacts on your device.

Social media platforms include:

- WhatsApp
- Facebook
- Instagram
- Twitter
- Discord
- GitHub
- GitLab
- Gravatar
- LinkedIn
- Twitch

The app will scrape the profile pics from the social media platforms and sync them into the contacts on your device.

Contacts will be imported from either user provided VCF files, Google Contacts, Outlook Contacts or CardDAV.

This app will have profiles system, to let the user manage their contacts from different sources.

Each profile will have their own dedicated cache sub folder in the app's data directory, to store the profile pics locally and other relevant files.
Contacts pic cache can be made reusable across profiles.

Any configuration settings, if any, will be per profile.

The contacts UI should should have search bar to search for contacts.

And the individual contact ui should have 3 tabbed sections:

1. Contact details
2. Contact editor
3. Profile pic selector

## Profile Pics extraction logic

For whatsapp, it can use the logic from the following repository:
<https://github.com/guyzyl/whatsapp-contact-sync>

For Gravatar, use the "email" field of the contact to extract the profile pic.

For the remaining:
The app should use the profile links present in the contacts itself to extract the profile pics.
Let the user populate the "website" field of contact with the respective profile link.
Add a convenience UI to let the user populate the "website" field of contact with the respective profile link in profile pic selector, which will update the contact's website field.

Contacts can have multiple website links, remember that.

## Backups

The app should support backups of contacts, so the user can restore their contacts from a backup file.

There should be VCF backup, import & export support.
And profile export/import support.

A scheduled backup support should be available, so the user can set up automatic backups at a specified directory along with internal backups directory.

It should also create a backup of contacts & profile before any write operation is done in any case.

## Sync

The app should be able to sync contacts back into:

- OS Contacts
- Google Contacts (by signing in)
- Outlook Contacts (by signing in)
- iCloud Contacts (by signing in)

The individual contact UI should have a "sync" button that allows the user to sync that specific contact.
