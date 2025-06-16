# 写在前面

这次选课也是全选上了，尽管还是保持着之前8线程与0.5s轮询的保守策略（1s16次），但是目前还是很够用的，不需要也最好不用太暴力去一秒钟上百次请求来整

选课刚开始的时候，网站会崩掉，这时会出现【请求错误】的提示，然后上面的请求次数会变得缓慢增加，这个时候稳住就行！一直挂着，后面网络恢复后它仍然是保持着轮询的

2025年6月16日我大概试了一下，应该还是好使的，所以就不做更新了（因为编译一次tauri应用就会占上20GB左右的磁盘空间，歇了）

考研结束后看怎样再维护一下了，不过，应该就是12月底或者26年的事情了，摸了

# Funky Lesson 自动选课应用

FunkyLesson是基于Leptos+Actix+Tauri的纯Rust生态实现的吉林大学抢课脚本！目前0.0.4版本的开箱即用exe可以在release里面找到：

点击[这里](https://github.com/ZoneHerobrine/funky-lesson/releases/tag/release)选择`funky-lesson.exe`直接下载下来，双击即可开启使用，无需任何环境配置，下面是使用演示GIF

目前默认是8线程独立轮询你的收藏课程列表（每个线程均从不同的课开始轮询，各态历经），请求间隔暂定是500ms。断网会提示请求失败，网络环境恢复后会直接自动重连继续循环选课。

核心库是[funky_lesson_core](https://github.com/ZoneHerobrine/funky_lesson_core),支持no-wasm和wasm两种特性，里面也提供了TUI的全功能实现，不需要使用GUI的同学也可以直接拉这个库下来然后`cargo run <username> <password> <batch_id> <is_loop>`一下

![funky-lesson的桌面端GIF演示，没显示的话检查一下网络环境或者用电脑浏览器打开](./funky-lesson.gif)


# 温馨提醒

程序不能保证100%抢中课，并不是运行脚本就能高枕无忧，那个选课服务器并不太稳定，严重的网络中断可能随时发生

如果脚本无响应，请不要放弃去用浏览器手动刷新拼运气选课

# 免责声明

- 本软件仅供学习和研究使用，请勿将其用于任何违反学校或相关法律法规的行为。
- 使用本软件所产生的一切后果均由用户自行承担，开发者不对任何因使用本软件造成的直接或间接损失负责。
- 用户在使用本软件的过程中，需遵守所在机构及国家的相关法律法规，如因使用本软件违反相关规定，责任由用户自行承担。
- 本软件未经吉林大学官方授权，与吉林大学无任何直接或间接关联。

如若使用本程序，即代表您同意本免责声明


# License

MIT License

# Acknowledgement

本项目核心库[funky_lesson_core](https://github.com/ZoneHerobrine/funky_lesson_core)的`no-wasm`特性部分是基于[MoonWX从H4ckF0rFun同学那里Fork下来的Fuck-Lesson](https://github.com/MoonWX/Fuck-Lesson)（一个python单文件抢课脚本）重写而成的（在examples文件夹下的standalone.rs包含了rust的单文件实现(但我去掉了ocr部分，感觉不太必要)，而src里面则是我封装和适配app之后的版本）

但无论是MoonWX同学还是H4ckF0rFun同学的Fuck-Lesson仓库都没有挂证书，只能在这里口头Acknowledgement了（

原python脚本原封不动放在[funky_lesson_core](https://github.com/ZoneHerobrine/funky_lesson_core)仓库的raw.py里面了


# 下面是一些神秘的开发日志

## 0.0.5 尝试安卓失败，明明android dev跑出来都好使，但是我android build成apk之后怎么手机上就一个请求都出不去

难绷，从九点半到十一点半挣了俩小时，配了四个版本都难绷不好使，最新的apk也贴release里了，不管了，我觉得可能是因为我手机的默认浏览器是火狐导致的QAQ（

有时间在修吧~，明天选课的话exe完全够用了


## 0.0.3 OK了，Proxy赢了，挺好的能用

先临时build一份推上去吧，还有很多warning，等晚上有时间再优化

0.0.4是0.1.0稳定版之前跑通的版本，先挂着吧

## 0.0.2 之前的战败宣言
实际上也彻底投降了，选课网站的CORS十分严格，Web应用无法直接访问选课网站的API。

上代理服务器试了一会儿，还没调通，但是感到很唐，因为服务端的请求本来就是已经写完了的，再写一个说实话和用tauri区别不大了

当初为了避开tauri，最主要的原因是因为tauri::command不支持流式传输，所以就想用wasm的网络请求库直接再leptos那边请求并且拿到响应

结果难绷了，因为CORS的问题，前端无法直接请求选课网站，这下过tauri或者过proxy都是要中介一个流了，proxy可能还稍微能实现一点实时性

唉，最后再试试吧，不行就火速切一下solid了事，也算是leptos的Web应用的又一次尝试了

---
并且Leptos对GRPC的支持也不够完善，用代理的话也没法很好的流式显示选课结果

我觉得更好的解决肯定还就直接上vite系的前端框架了，对grpc的支持更好

就可以通过core->grpc->ui的方式来实现选课的实时显示，验证码之类的倒是可以直接前端拿到，这个没CORS
