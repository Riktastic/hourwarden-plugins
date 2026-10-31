# Outlook mail

Reads the subject and the sender and recipient domains of the mail or appointment you have open or selected in Outlook (classic), so mail time goes to the customer with that mail domain. Also counts the mails you send to and receive from each customer.

## What it does

- Reads the subject and the sender's and recipients' mail domains of the mail or appointment you have open or selected in Outlook (classic), so time in Outlook goes to the customer with that mail domain.
- Counts the mails you send to and receive from each customer, shown on the customer page and in reports.

## Setting it up

1. Add the customer's **mail domains** on the customer page (`contoso.com`).
2. Add your own company's domains under *Ignore these domains*: they are in every mail and say nothing about the customer.

## What it reads

The subject, sender and recipients of the selected item, and the date, sender and recipients of mail in Sent Items and the Inbox. Never the body.

## When it does not work

- The new Outlook (olk.exe) has no automation interface; this plugin needs Outlook classic.
- When Outlook asks whether a program may read e-mail addresses, turn off *Count mails per customer* or allow it.

## Settings

| Setting | Default | What it does |
|---|---|---|
| Check every | `3` | How often the selected item is checked while Outlook is in front, in seconds. |
| Recipients to read | `30` | Mails to more people than this only use the first ones, to keep Outlook fast. |
| Count mails per customer | `on` | Count the mails in Sent Items and the Inbox per customer mail domain. Turn off if Outlook asks whether a program may read e-mail addresses. |
| Count new mails every | `5` | How often Sent Items and the Inbox are checked for new mail, in minutes. |
| Count mails from the last | `7` | How many days back mails are counted when the plugin starts. Mails already counted are not counted twice. |
| Ignore these domains | `(none)` | Your own company's mail domains, which appear in every mail and say nothing about the customer. |
