package metadata

generated: components: sources: stdin: configuration: {
	decoding: {
		description: """
			Configures how events are decoded from raw bytes. Note some decoders can also determine the event output
			type (log, metric, trace).
			"""
		required: false
		type: object: options: {
			avro: {
				description:   "Apache Avro-specific encoder options."
				relevant_when: "codec = \"avro\""
				required:      true
				type: object: options: {
					schema: {
						description: """
																The Avro schema definition.
																**Note**: The following [`apache_avro::types::Value`] variants are *not* supported:
																* `Date`
																* `Decimal`
																* `Duration`
																* `Fixed`
																* `TimeMillis`
																"""
						required: true
						type: string: examples: ["{ \"type\": \"record\", \"name\": \"log\", \"fields\": [{ \"name\": \"message\", \"type\": \"string\" }] }"]
					}
					strip_schema_id_prefix: {
						description: """
																For Avro datum encoded in Kafka messages, the bytes are prefixed with the schema ID.  Set this to `true` to strip the schema ID prefix.
																According to [Confluent Kafka's document](https://docs.confluent.io/platform/current/schema-registry/fundamentals/serdes-develop/index.html#wire-format).
																"""
						required: true
						type: bool: {}
					}
				}
			}
			codec: {
				description: "The codec to use for decoding events."
				required:    false
				type: string: {
					default: "bytes"
					enum: {
						avro: """
															Decodes the raw bytes as as an [Apache Avro][apache_avro] message.

															[apache_avro]: https://avro.apache.org/
															"""
						bytes: "Uses the raw bytes as-is."
						gelf: """
															Decodes the raw bytes as a [GELF][gelf] message.

															This codec is experimental for the following reason:

															The GELF specification is more strict than the actual Graylog receiver.
															Vector's decoder adheres more strictly to the GELF spec, with
															the exception that some characters such as `@`  are allowed in field names.

															Other GELF codecs such as Loki's, use a [Go SDK][implementation] that is maintained
															by Graylog, and is much more relaxed than the GELF spec.

															Going forward, Vector will use that [Go SDK][implementation] as the reference implementation, which means
															the codec may continue to relax the enforcement of specification.

															[gelf]: https://docs.graylog.org/docs/gelf
															[implementation]: https://github.com/Graylog2/go-gelf/blob/v2/gelf/reader.go
															"""
						influxdb: """
															Decodes the raw bytes as an [Influxdb Line Protocol][influxdb] message.

															[influxdb]: https://docs.influxdata.com/influxdb/cloud/reference/syntax/line-protocol
															"""
						json: """
															Decodes the raw bytes as [JSON][json].

															[json]: https://www.json.org/
															"""
						native: """
															Decodes the raw bytes as [native Protocol Buffers format][vector_native_protobuf].

															This decoder can output all types of events (logs, metrics, traces).

															This codec is **[experimental][experimental]**.

															[vector_native_protobuf]: https://github.com/vectordotdev/vector/blob/master/lib/vector-core/proto/event.proto
															[experimental]: https://vector.dev/highlights/2022-03-31-native-event-codecs
															"""
						native_json: """
															Decodes the raw bytes as [native JSON format][vector_native_json].

															This decoder can output all types of events (logs, metrics, traces).

															This codec is **[experimental][experimental]**.

															[vector_native_json]: https://github.com/vectordotdev/vector/blob/master/lib/codecs/tests/data/native_encoding/schema.cue
															[experimental]: https://vector.dev/highlights/2022-03-31-native-event-codecs
															"""
						protobuf: """
															Decodes the raw bytes as [protobuf][protobuf].

															[protobuf]: https://protobuf.dev/
															"""
						syslog: """
															Decodes the raw bytes as a Syslog message.

															Decodes either as the [RFC 3164][rfc3164]-style format ("old" style) or the
															[RFC 5424][rfc5424]-style format ("new" style, includes structured data).

															[rfc3164]: https://www.ietf.org/rfc/rfc3164.txt
															[rfc5424]: https://www.ietf.org/rfc/rfc5424.txt
															"""
						vrl: """
															Decodes the raw bytes as a string and passes them as input to a [VRL][vrl] program.

															[vrl]: https://vector.dev/docs/reference/vrl
															"""
					}
				}
			}
			gelf: {
				description:   "GELF-specific decoding options."
				relevant_when: "codec = \"gelf\""
				required:      false
				type: object: options: lossy: {
					description: """
						Determines whether to replace invalid UTF-8 sequences instead of failing.

						When true, invalid UTF-8 sequences are replaced with the [`U+FFFD REPLACEMENT CHARACTER`][U+FFFD].

						[U+FFFD]: https://en.wikipedia.org/wiki/Specials_(Unicode_block)#Replacement_character
						"""
					required: false
					type: bool: default: true
				}
			}
			influxdb: {
				description:   "Influxdb-specific decoding options."
				relevant_when: "codec = \"influxdb\""
				required:      false
				type: object: options: lossy: {
					description: """
						Determines whether to replace invalid UTF-8 sequences instead of failing.

						When true, invalid UTF-8 sequences are replaced with the [`U+FFFD REPLACEMENT CHARACTER`][U+FFFD].

						[U+FFFD]: https://en.wikipedia.org/wiki/Specials_(Unicode_block)#Replacement_character
						"""
					required: false
					type: bool: default: true
				}
			}
			json: {
				description:   "JSON-specific decoding options."
				relevant_when: "codec = \"json\""
				required:      false
				type: object: options: lossy: {
					description: """
						Determines whether to replace invalid UTF-8 sequences instead of failing.

						When true, invalid UTF-8 sequences are replaced with the [`U+FFFD REPLACEMENT CHARACTER`][U+FFFD].

						[U+FFFD]: https://en.wikipedia.org/wiki/Specials_(Unicode_block)#Replacement_character
						"""
					required: false
					type: bool: default: true
				}
			}
			native_json: {
				description:   "Vector's native JSON-specific decoding options."
				relevant_when: "codec = \"native_json\""
				required:      false
				type: object: options: lossy: {
					description: """
						Determines whether to replace invalid UTF-8 sequences instead of failing.

						When true, invalid UTF-8 sequences are replaced with the [`U+FFFD REPLACEMENT CHARACTER`][U+FFFD].

						[U+FFFD]: https://en.wikipedia.org/wiki/Specials_(Unicode_block)#Replacement_character
						"""
					required: false
					type: bool: default: true
				}
			}
			protobuf: {
				description:   "Protobuf-specific decoding options."
				relevant_when: "codec = \"protobuf\""
				required:      false
				type: object: options: {
					desc_file: {
						description: """
																The path to the protobuf descriptor set file.

																This file is the output of `protoc -I <include path> -o <desc output path> <proto>`

																You can read more [here](https://buf.build/docs/reference/images/#how-buf-images-work).
																"""
						required: false
						type: string: default: ""
					}
					message_type: {
						description: "The name of the message type to use for serializing."
						required:    false
						type: string: {
							default: ""
							examples: ["package.Message"]
						}
					}
				}
			}
			syslog: {
				description:   "Syslog-specific decoding options."
				relevant_when: "codec = \"syslog\""
				required:      false
				type: object: options: lossy: {
					description: """
						Determines whether to replace invalid UTF-8 sequences instead of failing.

						When true, invalid UTF-8 sequences are replaced with the [`U+FFFD REPLACEMENT CHARACTER`][U+FFFD].

						[U+FFFD]: https://en.wikipedia.org/wiki/Specials_(Unicode_block)#Replacement_character
						"""
					required: false
					type: bool: default: true
				}
			}
			vrl: {
				description:   "VRL-specific decoding options."
				relevant_when: "codec = \"vrl\""
				required:      true
				type: object: options: {
					source: {
						description: """
																The [Vector Remap Language][vrl] (VRL) program to execute for each event.
																Note that the final contents of the `.` target will be used as the decoding result.
																Compilation error or use of 'abort' in a program will result in a decoding error.

																[vrl]: https://vector.dev/docs/reference/vrl
																"""
						required: true
						type: string: {}
					}
					timezone: {
						description: """
																The name of the timezone to apply to timestamp conversions that do not contain an explicit
																time zone. The time zone name may be any name in the [TZ database][tz_database], or `local`
																to indicate system local time.

																If not set, `local` is used.

																[tz_database]: https://en.wikipedia.org/wiki/List_of_tz_database_time_zones
																"""
						required: false
						type: string: examples: ["local", "America/New_York", "EST5EDT"]
					}
				}
			}
		}
	}
	framing: {
		description: """
			Framing configuration.

			Framing handles how events are separated when encoded in a raw byte form, where each event is
			a frame that must be prefixed, or delimited, in a way that marks where an event begins and
			ends within the byte stream.
			"""
		required: false
		type: object: options: {
			character_delimited: {
				description:   "Options for the character delimited decoder."
				relevant_when: "method = \"character_delimited\""
				required:      true
				type: object: options: {
					delimiter: {
						description: "The character that delimits byte sequences."
						required:    true
						type: ascii_char: {}
					}
					max_length: {
						description: """
																The maximum length of the byte buffer.

																This length does *not* include the trailing delimiter.

																By default, there is no maximum length enforced. If events are malformed, this can lead to
																additional resource usage as events continue to be buffered in memory, and can potentially
																lead to memory exhaustion in extreme cases.

																If there is a risk of processing malformed data, such as logs with user-controlled input,
																consider setting the maximum length to a reasonably large value as a safety net. This
																ensures that processing is not actually unbounded.
																"""
						required: false
						type: uint: {}
					}
				}
			}
			chunked_gelf: {
				description:   "Options for the chunked GELF decoder."
				relevant_when: "method = \"chunked_gelf\""
				required:      false
				type: object: options: {
					decompression: {
						description: "Decompression configuration for GELF messages."
						required:    false
						type: string: {
							default: "Auto"
							enum: {
								Auto: "Automatically detect the decompression method based on the magic bytes of the message."
								Gzip: "Use Gzip decompression."
								None: "Do not decompress the message."
								Zlib: "Use Zlib decompression."
							}
						}
					}
					max_length: {
						description: """
																The maximum length of a single GELF message, in bytes. Messages longer than this length will
																be dropped. If this option is not set, the decoder does not limit the length of messages and
																the per-message memory is unbounded.

																**Note**: A message can be composed of multiple chunks and this limit is applied to the whole
																message, not to individual chunks.

																This limit takes only into account the message's payload and the GELF header bytes are excluded from the calculation.
																The message's payload is the concatenation of all the chunks' payloads.
																"""
						required: false
						type: uint: {}
					}
					pending_messages_limit: {
						description: """
																The maximum number of pending incomplete messages. If this limit is reached, the decoder starts
																dropping chunks of new messages, ensuring the memory usage of the decoder's state is bounded.
																If this option is not set, the decoder does not limit the number of pending messages and the memory usage
																of its messages buffer can grow unbounded. This matches Graylog Server's behavior.
																"""
						required: false
						type: uint: {}
					}
					timeout_secs: {
						description: """
																The timeout, in seconds, for a message to be fully received. If the timeout is reached, the
																decoder drops all the received chunks of the timed out message.
																"""
						required: false
						type: float: default: 5.0
					}
				}
			}
			length_delimited: {
				description:   "Options for the length delimited decoder."
				relevant_when: "method = \"length_delimited\""
				required:      true
				type: object: options: {
					length_field_is_big_endian: {
						description: "Length field byte order (little or big endian)"
						required:    false
						type: bool: default: true
					}
					length_field_length: {
						description: "Number of bytes representing the field length"
						required:    false
						type: uint: default: 4
					}
					length_field_offset: {
						description: "Number of bytes in the header before the length field"
						required:    false
						type: uint: default: 0
					}
					max_frame_length: {
						description: "Maximum frame length"
						required:    false
						type: uint: default: 8388608
					}
				}
			}
			method: {
				description: "The framing method."
				required:    true
				type: string: enum: {
					bytes:               "Byte frames are passed through as-is according to the underlying I/O boundaries (for example, split between messages or stream segments)."
					character_delimited: "Byte frames which are delimited by a chosen character."
					chunked_gelf: """
						Byte frames which are chunked GELF messages.

						[chunked_gelf]: https://go2docs.graylog.org/current/getting_in_log_data/gelf.html
						"""
					length_delimited:  "Byte frames which are prefixed by an unsigned big-endian 32-bit integer indicating the length."
					newline_delimited: "Byte frames which are delimited by a newline character."
					octet_counting: """
						Byte frames according to the [octet counting][octet_counting] format.

						[octet_counting]: https://tools.ietf.org/html/rfc6587#section-3.4.1
						"""
				}
			}
			newline_delimited: {
				description:   "Options for the newline delimited decoder."
				relevant_when: "method = \"newline_delimited\""
				required:      false
				type: object: options: max_length: {
					description: """
						The maximum length of the byte buffer.

						This length does *not* include the trailing delimiter.

						By default, there is no maximum length enforced. If events are malformed, this can lead to
						additional resource usage as events continue to be buffered in memory, and can potentially
						lead to memory exhaustion in extreme cases.

						If there is a risk of processing malformed data, such as logs with user-controlled input,
						consider setting the maximum length to a reasonably large value as a safety net. This
						ensures that processing is not actually unbounded.
						"""
					required: false
					type: uint: {}
				}
			}
			octet_counting: {
				description:   "Options for the octet counting decoder."
				relevant_when: "method = \"octet_counting\""
				required:      false
				type: object: options: max_length: {
					description: "The maximum length of the byte buffer."
					required:    false
					type: uint: {}
				}
			}
		}
	}
	host_key: {
		description: """
			Overrides the name of the log field used to add the current hostname to each event.

			By default, the [global `log_schema.host_key` option][global_host_key] is used.

			[global_host_key]: https://vector.dev/docs/reference/configuration/global-options/#log_schema.host_key
			"""
		required: false
		type: string: {}
	}
	max_length: {
		description: """
			The maximum buffer size, in bytes, of incoming messages.

			Messages larger than this are truncated.
			"""
		required: false
		type: uint: {
			default: 102400
			unit:    "bytes"
		}
	}
}
