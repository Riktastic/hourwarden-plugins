# Outlook calendar

**Works on:** Windows only: it reads the calendar of classic Outlook through COM, which only exists on Windows.

Reads your Outlook (classic) appointments of the last days, so the daily review can add meetings you attended away from the computer, for the customer of the attendees' mail domains or the customer named in the subject.

## What it does

Reads your Outlook (classic) appointments of the last days. The daily review then offers them under *From your calendar*, for the customer of the attendees' mail domains or the customer named in the subject. One click adds a meeting you attended away from the computer.

## Setting it up

Add mail domains and aliases on the customer pages, and your own company's domains under *Ignore these domains*.

## What it reads

Time, subject, location and attendees of your appointments. Never the body.

## Settings

| Setting | Default | What it does |
|---|---|---|
| Days back | `7` | How many days before today are offered in the daily review. |
| Read the calendar every | `15` | How often the calendar is read again, in minutes. |
| Skip appointments marked free | `on` | Appointments shown as Free (reminders, blocked travel time) are not offered. |
| Attendees to read | `50` | Large meetings only use the first attendees, to keep Outlook fast. |
| Ignore these domains | `(none)` | Your own company's mail domains, which are in every meeting and say nothing about the customer. |
