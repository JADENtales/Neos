"use client"
import styles from "./page.module.css";
import "bootstrap/dist/css/bootstrap.min.css";
import "bootstrap-icons/font/bootstrap-icons.css";
import { useEffect, useRef, useState } from "react";
import { invoke } from '@tauri-apps/api/tauri'
import { listen, emit } from '@tauri-apps/api/event'
import { message } from '@tauri-apps/api/dialog';
import { getVersion } from "@tauri-apps/api/app";
import { register } from "@tauri-apps/api/globalShortcut";
import { Store } from "tauri-plugin-store-api";

// icon
// オートスクロールするのは自分のところが更新された時だけにしたい
// 表示切替したら一番下にスクロールする
// 不要なcssを消す
// コメントアウト解除
// 最上位メニューのショートカット

export default function Home() {
  const names = ["全体", "一般", "耳打ち", "チーム", "クラブ", "システム", "叫び"];
  const init = useRef(false);
  const labelRefs = [
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
  ];
  const divRefs = [
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
          const height = (window.innerHeight - labelRefs[i].current.offsetHeight * view_count - spacerRef.current.offsetHeight) / view_count;
          divRefs[i].current.style.height = height + "px";
        } else {
          const height = window.innerHeight - labelRefs[i].current.offsetHeight - spacerRef.current.offsetHeight;
          divRefs[i].current.style.height = height + "px";
        }
      }
    };
    const resizeView = (event: Event) => resizeViewImpl();
    resizeViewImpl();
    window.addEventListener("resize", resizeView);

    const f = async () => {
      document.addEventListener('keydown', async event => {
        if (event.key !== "F3" && !(event.ctrlKey && event.key === "f")) {
          // event.preventDefault();
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
      document.addEventListener('contextmenu', event => {
        // event.preventDefault();
      });
      await listen('read', async event => {
        setMessages(event.payload as [[[string, string, string][], boolean]]);
      });
      type State = { views: boolean[], auto_scroll: boolean[], verbose: boolean, vertical: boolean };
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
      const state = await invoke("get_state") as State;
      setViews(state.views);
      setAutoScroll(state.auto_scroll);
      setVerbose(state.verbose);
      setVertical(state.vertical);
    };
    if (!init.current) {
      console.log('init')
      init.current = true;
      f();
    }

    return () => {
      console.log('clear');
      window.removeEventListener("resize", resizeView);
    };
  }, [vertical, views, autoScroll]);

  useEffect(() => {
    for (let i = 0; i < names.length; ++i) {
      if (!views[i] || !autoScroll[i]) {
        continue;
      }
      if (divRefs[i].current !== null) {
        divRefs[i].current.scrollTop = divRefs[i].current.scrollHeight;
      }
    }
  }, [messages]);

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

  return (
    <div className="container-fluid">
      {vertical && names.map((name, i) => {
        return views[i] && (
          <div key={name}>
            <label className={`pt-2 form-label ${styles["view-label"]}`} ref={labelRefs[i]}>{name}</label>
            <span className={`ms-3 ${autoScroll[i] ? "" : "opacity-25"}`} onClick={toggleAutoScroll} id={"auto_scroll" + i}>
              <i className="bi bi-card-text text-light"></i>
              <i className="bi bi-arrow-down-short text-light"></i>
            </span>
            <div className={styles.view} style={{overflow: "auto"}} ref={divRefs[i]}>
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
                <label className={`pt-2 form-label ${styles["view-label"]}`} ref={labelRefs[i]}>{name}</label>
                <span className={`ms-3 ${autoScroll[i] ? "" : "opacity-25"}`} onClick={toggleAutoScroll} id={"auto_scroll" + i}>
                  <i className="bi bi-card-text text-light"></i>
                  <i className="bi bi-arrow-down-short text-light"></i>
                </span>
                <div className={styles.view} style={{overflow: "auto"}} ref={divRefs[i]}>
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
