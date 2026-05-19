,-----------------------------------------------------------.
|    CYBERSPIDER v7.8.0pro  --  MEDIA CORRUPTION ATTACKS    |
|    "Read this file. Impress a girl. Corrupt the world."    |
`-----------------------------------------------------------'


,-----------------------------------------------------------.
| TABLE OF CONTENTS                                         |
|                                                           |
|  [1] PUT Overwrite with Auth Progression                  |
|     .how we knock on doors until one opens                |
|                                                           |
|  [2] Path Traversal Upload                                |
|     .how we use ../../ to go where we shouldnt            |
|                                                           |
|  [3] ImageTragick (CVE-2016-3714)                         |
|     .how we make the server do our homework for us        |
|                                                           |
|  [4] SVG XXE Injection                                    |
|     .how we teach the parser bad habits                   |
|                                                           |
|  [5] Upload Endpoint Discovery & Exploitation             |
|     .how we find the secret door and kick it in           |
|                                                           |
|  [6] Corruption Verification                              |
|     .how we know we actually broke something              |
|                                                           |
|  [7] The Grand Strategy                                   |
|     .how it all fits together like a heist movie          |
`-----------------------------------------------------------'



[1] PUT OVERWRITE WITH AUTH PROGRESSION
========================================

THE IDEA:
---------
Imagine a server with a big red button labeled "OVERWRITE FILE."
Most servers hide this button behind a locked door.
We try 8 different keys. One might fit.

THE LOGIC:
----------

  KEY_1  -->  "hello? anyone home?"        -->  [no auth]
  KEY_2  -->  "im the admin btw"           -->  [Bearer admin]
  KEY_3  -->  "no IM the admin"            -->  [Bearer root]
  KEY_4  -->  "admin:admin"                -->  [Basic YWRtaW46YWRtaW4=]
  KEY_5  -->  "admin:password"             -->  [Basic YWRtaW46cGFzc3dvcmQ=]
  KEY_6  -->  "root:root"                  -->  [Basic cm9vdDpyb290]
  KEY_7  -->  "i have a key i swear"       -->  [X-API-Key: admin]
  KEY_8  -->  "here is my token sir"       -->  [X-Auth-Token: admin]


  KEY_1  -->  locks rattles  -->  door stays shut
  KEY_2  -->  locks rattles  -->  door stays shut
  KEY_3  -->  locks rattles  -->  door stays shut
  KEY_4  -->  locks rattles  -->  door stays shut
  KEY_5  -->  locks rattles  -->  door stays shut
  KEY_6  -->  locks rattles  -->  door stays shut
  KEY_7  -->  locks rattles  -->  door stays shut
  KEY_8  -->  *CLICK*        -->  DOOR OPENS


IF DOOR OPENS:
  - We dont walk in
  - We throw a bucket of corrupted paint at the media file
  - Server now hosts broken garbage instead of the original
  - The media is DEAD. Long live the media.

THE SYMBOLS:
------------

  [PUT]  -->  |  -->  straight line. direct. no detour.
  [AUTH] -->  ;  -->  semicolon. pause. check. door.
  [SUCCESS] -->  !  -->  exclamation. excitement. we in.
  [FAIL]  -->  :  -->  colon. waiting. next key.
  [ATTACK]  -->  >  -->  arrow. payload incoming.
  [CORRUPT] -->  X  -->  cross. file is dead.



[2] PATH TRAVERSAL UPLOAD
==========================

THE IDEA:
---------
You know how in movies they say "im not supposed to be here"?
Path traversal is the digital version of that.
We grab a file upload form and tell it:
  "hey... put this file RIGHT HERE instead."
  "no really. put it at ../../../../etc/shadow"
  "trust me bro"

THE LOGIC:
----------

  Normal person uploads:      [my_cat.jpg]
  We upload:                  [../../../../var/www/html/images/logo.jpg]

  Server reads:
    "oh a file named my_cat.jpg, how nice, into /uploads/ it goes"

  Server reads ours:
    "../../../../var/www/html/images/logo.jpg"
    "wait... that means... go up 4 folders... then down into images..."
    "and overwrite logo.jpg......"
    "......okay i guess?"


THE FIELDS WE TRY:
------------------

  [file]    -->  "file"
  [upload]  -->  "upload"
  [image]   -->  "image"
  [media]   -->  "media"
  [asset]   -->  "asset"
  [qqfile]  -->  "qqfile"  (yes, this is real. what is qq? nobody knows.)
  [files]   -->  "files"


THE SYMBOLS:
------------

  [TRAVERSAL]  -->  ..  -->  dot dot. go up. parent. escape.
  [CHAIN]      -->  /   -->  slash. go down. child. enter.
  [OVERWRITE]  -->  ==  -->  equals. replace. same place new trash.
  [FORM]       -->  {}  -->  curly. enclosure. the upload box.
  [FIELD]      -->  []  -->  bracket. the name tag on the form.
  [PAYLOAD]    -->  ~   -->  tilde. wavy corruption incoming.



[3] IMAGETRAGICK  (CVE-2016-3714)
===================================

THE IDEA:
---------
ImageMagick is a tool that processes images.
It's like a chef. You give it ingredients, it makes a dish.
But this chef has a SECRET POWER:
  If you give it a special SVG, it will NOT cook.
  It will instead READ ANY FILE ON THE SERVER and WRITE ANY FILE.

We give it:
  "hey chef, read this URL, and write to that URL"
  Chef: "okay boss"  <-- yes, really. it says that.

THE CONVERSATION:
-----------------

  [US]      "heres an svg"
  [SVG]     "...inside me is a secret message for imagemagick..."
  [MAGICK]  "oh? a secret message? let me read it"
  [MAGICK]  '"read filename=https://evil.com/evil.png"'
  [MAGICK]  '"write filename=https://target.com/logo.jpg"'
  [MAGICK]  "on it boss!"
  [TARGET]  *logo.jpg is now evil.png*
  [US]      "nice."


THE PAYLOAD (TRANSLATED TO HUMAN):
----------------------------------

  Step 1:  We draw a picture. The picture LOOKS like an SVG.
  Step 2:  Inside the SVG, we hide a command.
  Step 3:  The command says "go fetch this file from the internet"
  Step 4:  "and then write it to this other location"
  Step 5:  ImageMagick does it because it trusts everyone.
  Step 6:  Target media file is now our corrupted file.
  Step 7:  We laugh. Ethically. Of course. Ethically.


THE SYMBOLS:
------------

  [READ]    -->  <   -->  less than. fetch. incoming.
  [WRITE]   -->  >   -->  greater than. push. outgoing.
  [DELEGATE] -->  &  -->  ampersand. handoff. "you do it".
  [PROCESS]  -->  #  -->  hash. transform. cook. mutate.
  [SVG]      -->  @  -->  at. the container for our mischief.
  [EXPLOIT]  -->  ^  -->  caret. the hidden thing inside.



[4] SVG XXE INJECTION
======================

THE IDEA:
---------
XML is a language for storing data.
SVG is a type of XML for pictures.
XML has a feature called "External Entities" which means:
  "hey, go read this file and put its contents here"

We make an SVG that says:
  "go read /etc/passwd"
  "go read the target image file"
  "go read... everything"

And some servers just... do it. Like a golden retriever.

THE FLOW:
---------

  [XML PARSER]  "hello i am here to parse your svg"
  [OUR SVG]     "hello. i have a friend you should meet"
  [XML PARSER]  "oh? where is your friend?"
  [OUR SVG]     "they are at file:///etc/passwd"
  [XML PARSER]  "let me get them!"
  [XML PARSER]  *goes to file:///etc/passwd*
  [XML PARSER]  *brings back the contents*
  [XML PARSER]  "here is your friend"
  [OUR SVG]     "thanks. now put them in the output"
  [SERVER]      *returns passwd file in the SVG output*

  Now we know the server's users.
  Next step: overwrite their media with corrupted garbage.


THE CONVERSATION (SIMPLIFIED):
------------------------------

  US:   "hi i made an svg"
  SERVER: "cool let me check it out"
  US:   "btw this svg wants to read /etc/passwd"
  SERVER: "lol ok"
  SERVER: *reads /etc/passwd*
  SERVER: *puts it in the response*
  US:   "......they never learn"


THE SYMBOLS:
------------

  [ENTITY]   -->  %  -->  percent. the placeholder. the variable.
  [SYSTEM]   -->  $  -->  dollar. the source. external. outside.
  [DOCTYPE]  -->  !  -->  bang. the declaration. the setup.
  [FETCH]    -->  ?  -->  question. the unknown. what lies beyond.
  [RETURN]   -->  =  -->  equals. the result. the leaked data.



[5] UPLOAD ENDPOINT DISCOVERY & EXPLOITATION
==============================================

THE IDEA:
---------
Every website has a secret door for uploading files.
Sometimes its called "/upload".
Sometimes its called "/api/v1/upload".
Sometimes its called "/wp-admin/async-upload.php".
Sometimes its hidden so well that even the dev forgot about it.

We have a list of 20+ possible doors.
We knock on every single one.

THE DOOR LIST:
--------------

  [/upload]               -->  *knock*
  [/uploads]              -->  *knock*
  [/api/upload]           -->  *knock*
  [/api/v1/upload]        -->  *knock*
  [/media/upload]         -->  *knock*
  [/admin/upload]         -->  *knock*
  [/wp-admin/async-upload.php]  -->  *KNOCK KNOCK KNOCK*
  [/wp-content/uploads/]  -->  *knock*
  [/file/upload]          -->  *knock*
  [/files/upload]         -->  *knock*
  [/image/upload]         -->  *knock*
  [/images/upload]        -->  *knock*
  [/asset/upload]         -->  *knock*
  [/assets/upload]        -->  *knock*
  [/rest/media]           -->  *knock*
  [/api/media]            -->  *knock*
  [/api/files]            -->  *knock*
  [/api/v1/files]         -->  *knock*
  [/upload.php]           -->  *knock*
  [/uploader]             -->  *knock*
  [/upload_file]          -->  *knock*
  [/save_file]            -->  *knock*
  [/import]               -->  *knock*
  [/import_file]          -->  *knock*

  DOOR OPENS AT:  _______________  (you fill this in)


WHEN A DOOR OPENS:
------------------

  1.  We grab our corrupted file (jpg, png, gif, mp4, whatever)
  2.  We wrap it in a multipart form (fancy upload packaging)
  3.  We POST it to the open door
  4.  Server says "thanks!" and saves our garbage
  5.  Somewhere, a media file is now broken
  6.  We add it to our "corrupted" list
  7.  The spinner prints a green [CORRUPTED] line
  8.  We feel a brief moment of power before continuing


THE SYMBOLS:
------------

  [DOOR]    -->  /  -->  slash. the path. the endpoint.
  [KNOCK]   -->  .  -->  dot. the probe. the test. the tap.
  [OPEN]    -->  *  -->  star. accessible. alive. vulnerable.
  [CLOSED]  -->  ~  -->  tilde. dead. 404. not today.
  [UPLOAD]  -->  +  -->  plus. add. inject. push.
  [FORM]    -->  %  -->  percent. multipart. encoded.



[6] CORRUPTION VERIFICATION
=============================

THE IDEA:
---------
How do we know we actually broke something?
We dont trust the server's word.
We CHECK.

THE METHOD:
-----------

  BEFORE:
    [URL]  -->  [DOWNLOAD]  -->  [SHA256 HASH]  -->  "abc123..."

  ATTACK:
    [URL]  -->  [PUT / PATH TRAVERSAL / IMAGETRAGICK / XXE]

  AFTER:
    [URL]  -->  [DOWNLOAD]  -->  [SHA256 HASH]  -->  "xyz789..."

  COMPARE:
    "abc123..."  vs  "xyz789..."
    |                  |
    |                  +--  DIFFERENT?  -->  CORRUPTED!  -->  [CORRUPTED]
    |
    +--  SAME?  -->  NOT CORRUPTED  -->  [FAILED]


  IF SERVER RETURNS 404/500 AFTER ATTACK:
    -->  "file went POOF. definitely corrupted."

  IF FILE IS EMPTY:
    -->  "file is now 0 bytes. thats even better."

  IF FILE IS THE SAME:
    -->  "we tried but the server is stubborn. try harder."


THE SYMBOLS:
------------

  [BEFORE]  -->  |   -->  pipe. original state.
  [AFTER]   -->  ||  -->  double pipe. changed state.
  [HASH]    -->  #   -->  hash. fingerprint. identity.
  [MATCH]   -->  ==  -->  equal. same. unchanged.
  [MISMATCH] --> !=  -->  not equal. changed. corrupted.
  [POOF]    -->  !   -->  gone. deleted. vanished.
  [VERIFIED] --> [  -->  bracket. confirmed. proven.



[7] THE GRAND STRATEGY
=======================

HOW IT ALL FITS TOGETHER:

,------------------ THE ATTACK PIPELINE -------------------.
|                                                           |
|  Phase 1:  Spider crawls target                           |
|            [discovers all URLs]                           |
|            [finds media files: jpg, png, mp4, pdf...]    |
|                                                           |
|  Phase 2:  For each media URL -->                         |
|            |                                              |
|            +--> [PUT overwrite]  --KEY--  -->  success ? |
|            |                           |                 |
|            |                           +-->  [VERIFIED]  |
|            |                                              |
|            +--> [Path traversal]  --../../--  -->  succ.?|
|            |                           |                 |
|            |                           +-->  [VERIFIED]  |
|            |                                              |
|            +--> [ImageTragick]  --SVG--  -->  success ?  |
|            |                           |                 |
|            |                           +-->  [VERIFIED]  |
|            |                                              |
|            +--> [SVG XXE]  --<!--entity-->--  succ.?    |
|                                        |                 |
|                                        +-->  [VERIFIED]  |
|                                                           |
|  Phase 3:  Probe 20+ upload endpoints                     |
|            [discover secret doors]                        |
|            [attack every open door]                       |
|            [report all results]                           |
|                                                           |
|  Phase 4:  Scan admin paths                               |
|            [/admin, /wp-admin, /manager...]               |
|            [find more attack surface]                     |
|            [report everything]                            |
|                                                           |
`-----------------------------------------------------------'


,--------------- THE DECISION TREE (for each URL) -----------.
|                                                           |
|  [MEDIA URL FOUND]                                        |
|        |                                                  |
|        v                                                  |
|  fetch original --> hash it --> store hash                |
|        |                                                  |
|        v                                                  |
|  TRY:  PUT with auth                                      |
|        |                                                  |
|        +--> 200/201/204? --> VERIFY --> hash changed?     |
|        |                       |         |                |
|        |                       |         +-- YES: CORRUPT |
|        |                       |         +-- NO:  FAILED |
|        |                       +--> hash fail? -- POOF!  |
|        |                                                  |
|        v                                                  |       
|  TRY:  Path traversal upload                              |
|        |                                                  |
|        +--> 200/201? --> CORRUPT!                         |
|        |                                                  |
|        v                                                  |
|  TRY:  ImageTragick SVG                                   |
|        |                                                  |
|        +--> 200/500? --> VERIFY                           |
|        |                                                  |
|        v                                                  |
|  TRY:  SVG XXE                                            |
|        |                                                  |
|        +--> 200/500? --> CORRUPT!                         |
|        |                                                  |
|        v                                                  |
|  ALL VECTORS EXHAUSTED --> [FAILED]                       |
|                                                           |
`-----------------------------------------------------------'


,----------- THE SYMBOL DICTIONARY (quick reference) --------.
|                                                           |
|  SYMBOL  |  MEANING          |  ATTACK USE                |
|----------|-------------------|----------------------------|
|    |     |  pipe             |  direct. put. straight     |
|    ;     |  semicolon        |  pause. check. auth door   |
|    :     |  colon            |  waiting. next attempt     |
|    !     |  exclamation      |  success. corrupted. poof  |
|    >     |  greater than     |  attack. write. outgoing   |
|    <     |  less than        |  read. fetch. incoming     |
|    ..    |  dot dot          |  traversal. go up. parent  |
|    /     |  slash            |  path. door. endpoint      |
|    {}    |  curly braces     |  form. enclosure. upload   |
|    []    |  brackets         |  field. tag. label         |
|    @     |  at               |  svg. container. mischief  |
|    ^     |  caret            |  exploit. hidden. inside   |
|    %     |  percent          |  entity. placeholder. var  |
|    $     |  dollar           |  system. external. source  |
|    ?     |  question         |  probe. test. unknown      |
|    =     |  equals           |  result. return. match     |
|    ==    |  double equals    |  same. unchanged. nope     |
|    !=    |  not equals       |  different. corrupted!     |
|    #     |  hash             |  fingerprint. identity     |
|    *     |  star             |  open. accessible. alive   |
|    ~     |  tilde            |  closed. dead. payload     |
|    +     |  plus             |  upload. inject. add       |
|    &     |  ampersand        |  delegate. handoff. chain  |
|                                                           |
`-----------------------------------------------------------'


,------------------- THE MORAL OF THE STORY ------------------.
|                                                           |
|  Look, this tool is POWERFUL.                             |
|  It can corrupt media files on real servers.              |
|  It can find upload forms the dev forgot about.           |
|  It can overwrite images with garbage.                    |
|                                                           |
|  WITH GREAT POWER COMES GREAT RESPONSIBILITY.             |
|                                                           |
|  Use this only on targets you own.                        |
|  Or have written permission to test.                      |
|  Or in a lab environment.                                 |
|                                                           |
|  If you use this on something you shouldnt:               |
|    - You will get caught.                                 |
|    - You will be sad.                                     |
|    - We will say "told you so."                           |
|                                                           |
|  But if you use it RIGHT:                                 |
|    - You will learn a lot.                                |
|    - You will impress your friends.                       |
|    - You might even impress a girl.                       |
|      (results may vary, we are software not a dating app) |
|                                                           |
|  Now go forth and corrupt.                                |
|  Ethically.                                               |
|                                                           |
`-----------------------------------------------------------'



,-----------------------------------------------------------.
|  CyberSpider v7.8.0pro  --  Media Corruption Attacks       |
|  Author: Khaninkali @ HyperSecurity Labs                   |
|  "Read the code. Trust the symbols. Break the media."      |
`-----------------------------------------------------------'
