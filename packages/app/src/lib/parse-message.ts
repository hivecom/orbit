import DOMPurify from "dompurify"
import { h, ref, type VNode } from "vue"
import ChatImage from "../components/shared/chat/ChatImage.vue"
import ChatExternalLink from "../components/shared/chat/ChatExternalLink.vue"

export const ALLOWED_TAGS = ["a", "abbr", "b", "blockquote", "br", "code", "del", "div", "em", "h1", "h2", "h3", "h4", "h5", "h6", "hr", "i", "img", "ins", "kbd", "li", "mark", "ol", "p", "pre", "q", "s", "small", "span", "strong", "sub", "sup", "table", "tbody", "td", "tfoot", "th", "thead", "tr", "u", "ul"]
export const ALLOWED_ATTR = ["href", "src", "alt", "title", "width", "height", "colspan", "rowspan", "start", "lang", "dir", "class"]

/**
 * Parser is used to convert chunks of plain irc messages into interactive UI
 * elements. Such as links, embed images or mentions. String goes in, string
 * including parsed HTML goes out
 */

const URL = /(?<!["'=])\bhttps?:\/\/[^\s<>"'`]+(?<![.,;:!?)\]}'"])/gi
const IMAGE = /(gif|jpe?g|tiff?|png|webp|bmp)$/i
const VIDEO = /(mp4|webpm)$/i
const AUDIO = /(mp3|wav|ogg)$/i

type Child = VNode | string

// Converts URL into a DOM element (or vue component) handler
function urlToNode(url: string): Child {
  // TODO: clicking an imaeg shows it in a modal
  // - same as with link, single modal and we just change the src of the image
  // - if a single message has multiple images in it, create a gallery + show arrows in the modal
  // TODO: mentions
  // TODO: server links
  // TODO: channel links
  // TODO: voice links
  // TODO: link embeds (og image / title etc)
  // TODO: spotify embeds (create a custom provider?)
  // TODO: custom video player
  // TODO: custom audio player
  if (IMAGE.test(url)) {
    return h(ChatImage, { src: url })
  }

  if (AUDIO.test(url)) {
    return h("audio", { controls: true, src: url })
  }

  const video = url.match(VIDEO)
  if (video) {
    return h("video", { controls: true }, [h("source", { src: url, type: `video/${video[0]}` })])
  }

  // Default back to <a />
  return h(ChatExternalLink, { href: url }, () => url)
}

// Splits string into either plain text nodes or parsed vue components
function splitText(text: string): Child[] {
  const pieces: Child[] = []
  let last = 0
  // Extracts URLs out of a string and either returns empty string nodes or
  // components retrieved from urlToNode
  for (const match of text.matchAll(URL)) {
    if (match.index > last) {
      // If a normal string was between the last URL and this
      pieces.push(text.slice(last, match.index))
    }

    pieces.push(urlToNode(match[0]))

    // Save the end of URL index for inbetween string
    last = match.index + match[0].length
  }

  // gap between last URL and end of the text
  if (last < text.length) {
    pieces.push(text.slice(last))
  }

  return pieces
}

// Sanitizes the message. Return it as a node hierarchy that is then walked and
// parsed based on if the node is just text node or not
export function parseMessage(text?: string | null): Child[] {
  if (!text) return []

  const sanitized = DOMPurify.sanitize(text, {
    ALLOWED_TAGS,
    ALLOWED_ATTR,
  })

  return splitText(sanitized)
}

// Track opened modal handlers for links / images etc. The idea is to ship
// global components so that message components do _not_ embed and import any
// components themselves
interface MessageModal {
  type: "img" | "link"
  payload: string
}

export const messageModalState = ref<MessageModal | null>(null)

export function openMsgModal(type: MessageModal["type"], payload: MessageModal["payload"]) {
  messageModalState.value = { type, payload }
}

export function closeMsgModal() {
  messageModalState.value = null
}

export function isMsgActive(type: MessageModal["type"]) {
  return !!(messageModalState.value && messageModalState.value.type === type)
}
