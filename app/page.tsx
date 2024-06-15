"use client"
import styles from "./page.module.css";
import "bootstrap/dist/css/bootstrap.min.css";
import "bootstrap-icons/font/bootstrap-icons.css";
import { useEffect, useRef, useState } from "react";
import { invoke } from '@tauri-apps/api/tauri'
import { listen, emit } from '@tauri-apps/api/event'
import { message } from '@tauri-apps/api/dialog';
import { getVersion } from "@tauri-apps/api/app";
import { Store } from "tauri-plugin-store-api";

export default function Home() {
  const names = ["全体", "一般", "耳打ち", "チーム", "クラブ", "システム", "叫び"];
  const init = useRef(false);
  const expRef = useRef((null as unknown) as HTMLDivElement);
  const labelRefs = [
    useRef((null as unknown) as HTMLSpanElement),
    useRef((null as unknown) as HTMLSpanElement),
    useRef((null as unknown) as HTMLSpanElement),
    useRef((null as unknown) as HTMLSpanElement),
    useRef((null as unknown) as HTMLSpanElement),
    useRef((null as unknown) as HTMLSpanElement),
    useRef((null as unknown) as HTMLSpanElement),
  ];
  const messageRefs = [
    useRef((null as unknown) as HTMLDivElement),
    useRef((null as unknown) as HTMLDivElement),
    useRef((null as unknown) as HTMLDivElement),
    useRef((null as unknown) as HTMLDivElement),
    useRef((null as unknown) as HTMLDivElement),
    useRef((null as unknown) as HTMLDivElement),
    useRef((null as unknown) as HTMLDivElement),
  ];
  const spacerRef = useRef((null as unknown) as HTMLDivElement);
  const [messages, setMessages] = useState<[[string, string, string][], boolean][]>([...Array(names.length)].map(_ => [[], false]));
  const [views, setViews] = useState([...Array(names.length)].map(_ => true));
  const [exp, setExp] = useState([0, 0, 0]);
  const [expVisible, setExpVisible] = useState(false);
  const [verbose, setVerbose] = useState(false);
  const [vertical, setVertical] = useState(true);
  const [autoScroll, setAutoScroll] = useState([...Array(names.length).map(_ => true)]);

  useEffect(() => {
    const resizeViewImpl = () => {
      const view_count = views.filter((e: any) => e).length;
      for (let i = 0; i < views.length; ++i) {
        if (!views[i]) {
          continue;
        }
        if (vertical) {
          const expHeight = expRef.current === null ? 0 : expRef.current.offsetHeight;
          const height = (window.innerHeight - expHeight - labelRefs[i].current.offsetHeight * view_count - spacerRef.current.offsetHeight) / view_count;
          messageRefs[i].current.style.height = height + "px";
        } else {
          const expHeight = expRef.current === null ? 0 : expRef.current.offsetHeight;
          const height = window.innerHeight - expHeight - labelRefs[i].current.offsetHeight - spacerRef.current.offsetHeight;
          messageRefs[i].current.style.height = height + "px";
        }
      }
    };
    const resizeView = (event: Event) => resizeViewImpl();
    resizeViewImpl();
    window.addEventListener("resize", resizeView);

    const f = async () => {
      document.addEventListener('contextmenu', event => {
        event.preventDefault();
      });
      document.addEventListener('keydown', async event => {
        if (event.key !== "F3" && !(event.ctrlKey && event.key === "f")) {
          event.preventDefault();
        }
        for (let i = 0; i < names.length; ++i) {
          if (!event.ctrlKey && !event.shiftKey && !event.altKey && event.key === (i + 1).toString() && !event.repeat) {
            const store_name = await invoke("get_store_name") as string;
            const store = new Store(store_name);
            await store.load();
            let value = !await store.get("view" + i);
            await store.set("view" + i, value);
            await store.save();
            await emit("view_back" + i, value);
            setViews(prev => prev.map((e, j) => j === i ? !e : e));
          }
          if (event.ctrlKey && !event.shiftKey && !event.altKey && event.key === (i + 1).toString() && !event.repeat) {
            const store_name = await invoke("get_store_name") as string;
            const store = new Store(store_name);
            await store.load();
            let value = !await store.get("auto_scroll" + i);
            await store.set("auto_scroll" + i, value);
            await store.save();
            await emit("auto_scroll_back" + i, value);
            setAutoScroll(prev => prev.map((e, j) => j === i ? !e : e));
          }
        }
        if (!event.ctrlKey && !event.shiftKey && !event.altKey && event.key === "0" && !event.repeat) {
          const store_name = await invoke("get_store_name") as string;
          const store = new Store(store_name);
          await store.load();
          let value = !await store.get("exp");
          await store.set("exp", value);
          await store.save();
          await emit("exp_visible_back", value);
          setExpVisible(value);
        }
        if (!event.ctrlKey && !event.shiftKey && !event.altKey && event.key === "t" && !event.repeat) {
          const store_name = await invoke("get_store_name") as string;
          const store = new Store(store_name);
          await store.load();
          let value = !await store.get("verbose");
          await store.set("verbose", value);
          await store.save();
          await emit("verbose_back", value);
          setVerbose(value);
        }
        if (!event.ctrlKey && !event.shiftKey && !event.altKey && event.key === "d" && !event.repeat) {
          const store_name = await invoke("get_store_name") as string;
          const store = new Store(store_name);
          await store.load();
          let value = !await store.get("vertical");
          await store.set("vertical", value);
          await store.save();
          await emit("vertical_back", value);
          setVertical(value);
        }
      });
      await listen('read', async event => {
        setMessages(event.payload as [[[string, string, string][], boolean]]);
      });
      type State = { views: boolean[], exp: boolean, auto_scroll: boolean[], verbose: boolean, vertical: boolean };
      for (let i = 0; i < names.length; ++i) {
        await listen('view' + i, async event => {
          const state = await invoke("get_state") as State;
          setViews(state.views);
        });
        await listen('auto_scroll' + i, async event => {
          const state = await invoke("get_state") as State;
          setAutoScroll(state.auto_scroll);
        });
      }
      await listen('exp', async event => {
        setExp(event.payload as [number, number, number]);
      });
      await listen('exp_visible', async event => {
        const state = await invoke("get_state") as State;
        setExpVisible(state.exp);
      });
      await listen('verbose', async event => {
        const state = await invoke("get_state") as State;
        setVerbose(state.verbose);
      });
      await listen('vertical', async event => {
        const state = await invoke("get_state") as State;
        setVertical(state.vertical);
      });
      await listen('about', async event => {
        const version = await getVersion();
        await message(`バージョン: ${version}\n開発者X: @JADEN_tales`, { title: "Neosについて" });
      });
      await listen('error', async event => {
        await message("エラーが発生しました。ソフトを再起動してください。", { title: "エラー", type: "error" });
      });
      const state = await invoke("get_state") as State;
      setExpVisible(state.exp);
      setViews(state.views);
      setAutoScroll(state.auto_scroll);
      setVerbose(state.verbose);
      setVertical(state.vertical);
    };
    if (!init.current) {
      init.current = true;
      f();
    }

    return () => {
      window.removeEventListener("resize", resizeView);
    };
  }, [expVisible, vertical, views, autoScroll]);

  useEffect(() => {
    for (let i = 0; i < names.length; ++i) {
      if (!views[i] || !autoScroll[i] || !messages[i][1]) {
        continue;
      }
      if (messageRefs[i].current !== null) {
        messageRefs[i].current.scrollTop = messageRefs[i].current.scrollHeight;
      }
    }
  }, [messages]);

  useEffect(() => {
    for (let i = 0; i < names.length; ++i) {
      if (!views[i]) {
        continue;
      }
      if (messageRefs[i].current !== null) {
        messageRefs[i].current.scrollTop = messageRefs[i].current.scrollHeight;
      }
    }
  }, [vertical]);

  const toggleAutoScroll = async (event: React.MouseEvent<HTMLDivElement>) => {
    const i = parseInt(event.currentTarget.id[event.currentTarget.id.length - 1]);
    const store_name = await invoke("get_store_name") as string;
    const store = new Store(store_name);
    await store.load();
    const value = !autoScroll[i];
    await store.set("auto_scroll" + i, value);
    await store.save();
    await emit("auto_scroll_back" + i, value);
    setAutoScroll(prev => prev.map((e, j) => i === j ? !e : e));
  };

  const toCommaString = (value: number): string => {
    const s = value.toString().split("").reverse().join("");
    const commaNum = Math.trunc((s.length - 1) / 3);
    let r = "";
    for (let i = 0; i < commaNum; ++i) {
      r += s.substring(i * 3, i * 3 + 3) + ",";
    }
    r += s.substring(commaNum * 3);
    return r.split("").reverse().join("");
  };

  return (
    <div className="container-fluid">
      {
        expVisible && (
          <div ref={expRef}>
            <div className={`pt-1 pb-1 ${styles["view-label"]}`}>経験値</div>
            <div className={styles.exp}>{toCommaString(exp[0])}/秒</div>
            <div className={styles.exp}>{toCommaString(exp[1])}/分</div>
            <div className={styles.exp}>{toCommaString(exp[2])}/時</div>
          </div>
        )
      }
      {vertical && names.map((name, i) => {
        return views[i] && (
          <div key={name}>
            <span className={`d-inline-block pt-1 pb-1 ${styles["view-label"]}`} ref={labelRefs[i]}>{name}</span>
            <span className={`ms-3 ${autoScroll[i] ? "" : "opacity-25"}`} onClick={toggleAutoScroll} id={"auto_scroll" + i}>
              <i className="bi bi-card-text text-light"></i>
              <i className="bi bi-arrow-down-short text-light"></i>
            </span>
            <div className={styles.view} style={{overflow: "auto"}} ref={messageRefs[i]}>
              {
                messages[i][0].map((e, j) => {
                  const message = verbose ? e[2] + " " + e[0] : e[0];
                  return <div key={j} style={{color: e[1]}}>{message}</div>;
                })
              }
            </div>
          </div>
        );
      })}
      {!vertical &&
        <div className={`row row-cols-${views.filter(e => e).length} gx-2`}>
          {names.map((name, i) => {
            return views[i] && (
              <div className="col" key={name}>
                <div>
                  <span className={`d-inline-block pt-1 pb-1 ${styles["view-label"]}`} ref={labelRefs[i]}>{name}</span>
                  <span className={`ms-3 ${autoScroll[i] ? "" : "opacity-25"}`} onClick={toggleAutoScroll} id={"auto_scroll" + i}>
                    <i className="bi bi-card-text text-light"></i>
                    <i className="bi bi-arrow-down-short text-light"></i>
                  </span>
                </div>
                <div className={styles.view} style={{overflow: "auto"}} ref={messageRefs[i]}>
                  {
                    messages[i][0].map((e, j) => {
                      const message = verbose ? e[2] + " " + e[0] : e[0];
                      return <div key={j} style={{color: e[1]}}>{message}</div>;
                    })
                  }
                </div>
              </div>
            );
          })}
        </div>
      }
      <div className="invisible p-1" ref={spacerRef}></div>
    </div>
  );
}
